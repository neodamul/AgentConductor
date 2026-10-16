use super::{Pending, Probe, room};
use eframe::egui::{self, Color32, CornerRadius, Frame, vec2};
use serde_json::Value;

mod execution;
mod goal;
pub mod header;
pub mod mcp;
pub mod messenger;
pub mod navigation;
mod permissions;
mod planning;
pub mod profile_tools;
mod review;
pub mod sharing;
pub mod sidebar;
pub(super) mod tasks;
mod team;
pub use execution::execution_ready;
pub use execution::{Event as StartEvent, controls as start_controls, start_request};
pub use planning::{planning_activity_color, planning_status};
pub use role_rules::show_in_language as show_role_rules_in_language;
pub use tasks::{
    Draft as TaskDraft, Event as TaskEvent, controls as task_controls,
    controls_in_language as task_controls_in_language,
};
pub use team::profile_from_template;
mod role_rules;
pub use role_rules::show as show_role_rules;
pub fn prepare_request(revision: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"kind":"prepare","revision":revision}))
        .expect("prepare request is serializable")
}

pub(super) fn show(probe: &mut Probe, ui: &mut egui::Ui) {
    let background = ui.visuals().panel_fill;
    ui.painter().rect_filled(ui.max_rect(), 0.0, background);
    ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
    Frame::new()
        .fill(background)
        .inner_margin(24.0)
        .show(ui, |ui| {
            probe.show_navigation(ui);
            if let Some(mcp::Event::Save(settings)) = probe.mcp_editor.show(
                probe.snapshot.as_ref(),
                probe.navigation.appearance.language(),
                permissions::editable(probe) && probe.room_view.is_none(),
                ui,
            ) {
                probe.save_mcp_settings(&settings);
            }
            if let Some(event) = probe.messenger_editor.show(
                probe.snapshot.as_ref().unwrap_or(&Value::Null),
                probe.navigation.appearance.language(),
                permissions::editable(probe)
                    && probe.room_view.is_none()
                    && probe.pending.is_none(),
                ui,
            ) {
                probe.send_messenger(event);
            }
            if let Some(room_view) = probe.room_view.as_mut() {
                if let Some(event) = room::show(
                    room_view,
                    probe.snapshot.as_ref(),
                    probe.pending.is_some(),
                    probe.error.as_deref(),
                    probe.navigation.appearance.language(),
                    ui,
                ) {
                    match event {
                        room::Event::Refresh => probe.refresh_room(),
                        room::Event::Stop(reason) => probe.stop_room(reason),
                        room::Event::Evidence => probe.load_evidence(),
                        room::Event::Decision(decision, reason) => {
                            probe.decide_room(decision, reason)
                        }
                        room::Event::Resume => probe.resume_room(),
                        room::Event::Recover => probe.recover_room(),
                        room::Event::ToolDecision(action, id) => probe.decide_tool(action, id),
                        room::Event::Briefing => probe.load_briefing(),
                        room::Event::TaskDetails(index) => probe.load_task_details(index),
                        room::Event::TaskAction(action) => probe.send_task_action(action),
                    }
                }
                return;
            }
            let event = egui::Panel::left("native-agent-roster")
                .resizable(true)
                .default_size(248.0)
                .size_range(210.0..=320.0)
                .show(ui, |ui| {
                    sidebar::show(
                        &probe.profiles,
                        probe.snapshot.as_ref().unwrap_or(&Value::Null),
                        probe.navigation.appearance.language(),
                        permissions::editable(probe),
                        ui,
                    )
                })
                .inner;
            match event {
                Some(sidebar::Event::AddAgent) => probe.workspace_step = 1,
                Some(sidebar::Event::EditAgent(id)) => {
                    if let Some(profile) = probe.profiles.iter().find(|item| item["id"] == id) {
                        probe.profile_editor.open(profile);
                        probe.workspace_step = 1;
                    }
                }
                Some(sidebar::Event::Quota(id)) => {
                    probe.request_metadata(profile_tools::Event::Quota(id));
                }
                None => {}
            }
            egui::CentralPanel::default().show(ui, |ui| {
                if let Some(step) = header::show(
                    probe.workspace_step,
                    probe.navigation.appearance.language(),
                    ui,
                ) {
                    probe.workspace_step = step;
                }
                if probe.room_view.is_none()
                    && let Some(event) =
                        super::cache::show(&mut probe.cache, ui, probe.pending.is_some())
                {
                    match event {
                        super::cache::Event::Preview => probe.preview_cache(),
                        super::cache::Event::Clear => probe.clear_cache(),
                    }
                }
                let editable = permissions::editable(probe);
                match probe.workspace_step {
                    0 => {
                        goal::show(probe, ui);
                        probe.show_sharing(ui);
                    }
                    1 => {
                        let snapshot = probe.snapshot.clone();
                        let event = snapshot
                            .as_ref()
                            .and_then(|snapshot| team::show(probe, editable, snapshot, ui));
                        if let Some(event) = event {
                            match event {
                                profile_tools::Event::Discover(_)
                                | profile_tools::Event::Quota(_) => probe.request_metadata(event),
                                profile_tools::Event::SaveProfiles => probe.send_profile_save(),
                                profile_tools::Event::BrowseExecutable => probe.pick_executable(),
                                profile_tools::Event::ResolveExecutable(path) => {
                                    probe.resolve_executable(path)
                                }
                            }
                        }
                        if ui
                            .add_enabled(editable, egui::Button::new("Save agent roles"))
                            .clicked()
                        {
                            probe.send_workspace();
                        }
                    }
                    _ => review::show(probe, ui),
                }
                if probe.pending.is_some() || probe.starting_run.is_some() {
                    ui.spinner();
                    let activity = if probe.starting_run.is_some() {
                        "start"
                    } else {
                        match probe.pending {
                            Some(Pending::PickFolder) => "folder",
                            Some(Pending::Prepare) => "prepare",
                            Some(Pending::Metadata) => "metadata",
                            Some(Pending::MetadataRefresh) => "metadata_refresh",
                            Some(Pending::Start) => "start",
                            Some(Pending::Refresh)
                                if probe
                                    .snapshot
                                    .as_ref()
                                    .is_some_and(|s| s["job"]["state"] == "running") =>
                            {
                                "plan_refresh"
                            }
                            Some(Pending::Navigation | Pending::NavigationRefresh) => "workspace",
                            Some(Pending::McpSave) => "mcp_save",
                            _ => "save",
                        }
                    };
                    ui.label(if activity == "mcp_save" {
                        probe.navigation.appearance.text("mcp.saving_mcp_settings")
                    } else if let Some(id) = match probe.pending {
                        Some(Pending::MessengerLoad) => Some("startup.loading_messenger_settings"),
                        Some(Pending::MessengerTest) => Some("startup.sending_test_message"),
                        Some(Pending::MessengerSave | Pending::MessengerRefresh) => {
                            Some("startup.saving_messenger_settings")
                        }
                        _ => None,
                    } {
                        probe.navigation.appearance.text(id)
                    } else {
                        pending_message(activity)
                    });
                }
                if let Some(error) = &probe.error {
                    ui.colored_label(Color32::DARK_RED, error);
                }
            });
        });
}

pub fn pending_message(kind: &str) -> &'static str {
    match kind {
        "folder" => "Choosing project folder…",
        "prepare" => "Starting plan generation…",
        "metadata" => "Queueing agent metadata refresh…",
        "metadata_refresh" => "Refreshing agent metadata…",
        "start" => "Starting agent execution…",
        "plan_refresh" => "Refreshing plan progress…",
        "workspace" => "Updating workspaces…",
        _ => "Saving workspace…",
    }
}

pub(super) fn card(ui: &egui::Ui) -> Frame {
    Frame::new()
        .fill(ui.visuals().window_fill)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(16.0)
}

pub fn task_edit_request(
    snapshot: &Value,
    tasks: &[Value],
    workspace_mode: crate::plan::WorkspaceMode,
    digest: &str,
) -> Result<Vec<u8>, String> {
    tasks::edit_request(snapshot, tasks, workspace_mode, digest)
}

impl Probe {
    pub(super) fn save_task_edits(
        &mut self,
        tasks: Vec<Value>,
        mode: crate::plan::WorkspaceMode,
        digest: &str,
    ) {
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Workspace proposal is unavailable".to_owned())
            .and_then(|snapshot| task_edit_request(snapshot, &tasks, mode, digest))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(Pending::Edit);
            }
            Err(error) => self.error = Some(error),
        }
    }
}

pub fn workspace_save_request(
    snapshot: &Value,
    name: &str,
    repository: &str,
    goal: &str,
    workspace_mode: crate::plan::WorkspaceMode,
    profiles: &[Value],
    coordinator: &str,
) -> Vec<u8> {
    let mut config = snapshot["config"].clone();
    config["name"] = name.into();
    config["repository"] = repository.into();
    config["goal"] = goal.into();
    config["workspace_mode"] = serde_json::to_value(workspace_mode).unwrap();
    config["profiles"] = serde_json::to_value(profiles).unwrap();
    config["coordinator"] = coordinator.into();
    serde_json::to_vec(&serde_json::json!({
        "kind": "save",
        "revision": snapshot["revision"],
        "config": config
    }))
    .expect("workspace save request is serializable")
}

pub fn picked_folder(current: &str, response: &Value) -> String {
    response["picked_folder"]
        .as_str()
        .unwrap_or(current)
        .to_owned()
}
