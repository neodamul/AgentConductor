use super::{Pending, Probe, room};
use eframe::egui::{self, Color32, Frame, vec2};
use serde_json::Value;

mod execution;
pub mod navigation;
mod planning;
pub mod profile_tools;
mod surface;
pub(super) mod tasks;
pub use execution::{Event as StartEvent, controls as start_controls, start_request};
pub use planning::{planning_activity_color, planning_status};
use surface::{editable, show_review, show_team, show_workspace};
pub use tasks::{Draft as TaskDraft, Event as TaskEvent, controls as task_controls};
mod role_rules;
pub use role_rules::show as show_role_rules;

pub fn prepare_request(revision: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"kind":"prepare","revision":revision}))
        .expect("prepare request is serializable")
}

pub(super) fn show(probe: &mut Probe, ui: &mut egui::Ui) {
    let background = Color32::from_rgb(242, 242, 246);
    ui.painter().rect_filled(ui.max_rect(), 0.0, background);
    ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
    Frame::new()
        .fill(background)
        .inner_margin(24.0)
        .show(ui, |ui| {
            probe.show_navigation(ui);
            if let Some(room_view) = probe.room_view.as_mut() {
                if let Some(event) = room::show(
                    room_view,
                    probe.snapshot.as_ref(),
                    probe.pending.is_some(),
                    probe.error.as_deref(),
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
                    }
                }
                return;
            }
            ui.horizontal(|ui| {
                for (index, label) in [
                    "1. Workspace settings",
                    "2. Agent roles",
                    "3. Review the plan",
                ]
                .iter()
                .enumerate()
                {
                    ui.selectable_value(&mut probe.workspace_step, index, *label)
                        .accessible_name(*label);
                }
            });
            if probe.room_view.is_none()
                && let Some(event) =
                    super::cache::show(&mut probe.cache, ui, probe.pending.is_some())
            {
                match event {
                    super::cache::Event::Preview => probe.preview_cache(),
                    super::cache::Event::Clear => probe.clear_cache(),
                }
            }
            let editable = editable(probe);
            match probe.workspace_step {
                0 => show_workspace(probe, ui),
                1 => {
                    let snapshot = probe.snapshot.clone();
                    let event = snapshot
                        .as_ref()
                        .and_then(|snapshot| show_team(probe, editable, snapshot, ui));
                    if let Some(event) = event {
                        match event {
                            profile_tools::Event::Discover(_) | profile_tools::Event::Quota(_) => {
                                probe.request_metadata(event)
                            }
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
                _ => show_review(probe, ui),
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
                        _ => "save",
                    }
                };
                ui.label(pending_message(activity));
            }
            if let Some(error) = &probe.error {
                ui.colored_label(Color32::DARK_RED, error);
            }
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

pub fn execution_ready(snapshot: &Value, expected_run: &str) -> bool {
    !expected_run.is_empty()
        && snapshot["run_available"] == true
        && snapshot["proposal"]["run"].as_str() == Some(expected_run)
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

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
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

pub fn profile_from_template(template: &Value, profiles: &[Value]) -> Option<Value> {
    if profiles.len() >= 16 {
        return None;
    }
    let base = template["id"].as_str()?;
    let mut id = base.to_owned();
    let mut suffix = 2;
    while profiles.iter().any(|profile| profile["id"] == id) {
        id = format!("{base}-{suffix}");
        suffix += 1;
    }
    let mut profile = template.clone();
    profile["id"] = id.into();
    Some(profile)
}

pub fn picked_folder(current: &str, response: &Value) -> String {
    response["picked_folder"]
        .as_str()
        .unwrap_or(current)
        .to_owned()
}
