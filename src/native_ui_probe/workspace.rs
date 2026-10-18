use super::{Pending, Probe, room};
use eframe::egui::{self, Color32, CornerRadius, Frame, vec2};
use serde_json::Value;

mod execution;
mod goal;
pub mod goals;
pub mod header;
pub mod mcp;
pub mod messenger;
pub mod navigation;
mod pending;
mod permissions;
mod planning;
pub mod profile_tools;
mod requests;
mod review;
mod room_events;
pub mod sharing;
pub mod sidebar;
pub(super) mod tasks;
mod team;
pub use execution::execution_ready;
pub use execution::{
    Event as StartEvent, controls as start_controls,
    controls_in_language as start_controls_in_language, start_request,
};
pub use pending::pending_message;
pub use planning::{planning_activity_color, planning_status};
pub use requests::{picked_folder, prepare_request, task_edit_request, workspace_save_request};
pub use role_rules::show_in_language as show_role_rules_in_language;
pub use role_rules::suggested as suggested_role_rules;
pub use tasks::{
    Draft as TaskDraft, Event as TaskEvent, controls as task_controls,
    controls_in_language as task_controls_in_language,
};
pub use team::profile_from_template;
mod role_rules;
mod scroll;
pub use role_rules::show as show_role_rules;
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
            if probe.room_view.is_some() {
                let event = {
                    let room_view = probe.room_view.as_mut().unwrap();
                    room::show(
                        room_view,
                        probe.snapshot.as_ref(),
                        probe.pending.is_some(),
                        probe.error.as_deref(),
                        probe.navigation.appearance.language(),
                        ui,
                    )
                };
                if let Some(event) = event {
                    room_events::apply(probe, event);
                }
                if probe.room_view.is_some() {
                    return;
                }
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
                        probe.profile_editor.is_editing(),
                        ui,
                    )
                })
                .inner;
            if let Some(event) = event {
                team::handle_event(probe, event);
            }
            egui::CentralPanel::default().show(ui, |ui| {
                if let Some(step) = header::show(
                    probe.workspace_step,
                    probe.navigation.appearance.language(),
                    ui,
                ) {
                    probe.workspace_step = step;
                }
                scroll::show(ui, |ui| {
                    let phase_event = probe.snapshot.as_ref().and_then(|snapshot| {
                        goals::show(
                            snapshot,
                            probe.navigation.appearance.language(),
                            probe.pending.is_some(),
                            ui,
                        )
                    });
                    goals::apply(probe, phase_event);
                    if probe.room_view.is_none()
                        && let Some(event) = probe.show_cache_panel(ui)
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
                                    | profile_tools::Event::Quota(_) => {
                                        probe.request_metadata(event)
                                    }
                                    profile_tools::Event::SetCatalog { agent, catalog } => {
                                        probe.save_catalog(&agent, &catalog)
                                    }
                                    profile_tools::Event::SaveProfiles => probe.send_profile_save(),
                                    profile_tools::Event::BrowseExecutable => {
                                        probe.pick_executable()
                                    }
                                    profile_tools::Event::ResolveExecutable(path) => {
                                        probe.resolve_executable(path)
                                    }
                                }
                            }
                            if ui
                                .add_enabled(
                                    editable,
                                    egui::Button::new(
                                        probe
                                            .navigation
                                            .appearance
                                            .text("startup.save_agent_roles"),
                                    ),
                                )
                                .accessible_name(
                                    probe.navigation.appearance.text("startup.save_agent_roles"),
                                )
                                .clicked()
                            {
                                probe.send_workspace();
                            }
                            let current = review::settings_are_current(
                                probe,
                                snapshot.as_ref().unwrap_or(&Value::Null),
                            );
                            let planning = snapshot.as_ref().is_some_and(|snapshot| {
                                snapshot["job"]["kind"] == "plan"
                                    && snapshot["job"]["state"] == "running"
                            });
                            let can_generate =
                                editable && current && !planning && probe.pending.is_none();
                            if ui
                                .add_enabled(
                                    can_generate,
                                    egui::Button::new(
                                        probe.navigation.appearance.text("startup.generate_plan"),
                                    ),
                                )
                                .accessible_name(
                                    probe.navigation.appearance.text("startup.generate_plan"),
                                )
                                .clicked()
                            {
                                probe.workspace_step = 2;
                                probe.prepare_plan();
                            }
                            if !current {
                                ui.label(
                                    probe
                                        .navigation
                                        .appearance
                                        .text("startup.review_save_before_generate"),
                                );
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
                                Some(Pending::Prepare | Pending::NextPhase) => "prepare",
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
                                Some(Pending::Navigation | Pending::NavigationRefresh) => {
                                    "workspace"
                                }
                                Some(Pending::SelectGoal) => "workspace",
                                Some(Pending::McpSave) => "mcp_save",
                                _ => "save",
                            }
                        };
                        ui.label(if activity == "mcp_save" {
                            probe.navigation.appearance.text("mcp.saving_mcp_settings")
                        } else if let Some(id) = match probe.pending {
                            Some(Pending::MessengerLoad) => {
                                Some("startup.loading_messenger_settings")
                            }
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
        });
}

pub(super) fn card(ui: &egui::Ui) -> Frame {
    Frame::new()
        .fill(ui.visuals().window_fill)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(16.0)
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
