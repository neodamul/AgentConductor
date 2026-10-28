use eframe::egui;
use serde_json::Value;
use std::sync::Arc;

use super::{Pending, Probe, RequestWorker, Surface, render, room, workspace};

mod background;
mod catalog;

impl Probe {
    pub fn connect_surface(
        room: Arc<crate::ui::Room>,
        context: egui::Context,
        surface: Surface,
    ) -> Result<Self, String> {
        match surface {
            Surface::Workspace => Self::connect(room, context),
            Surface::AgentRoom => Self::connect_room(room, context),
        }
    }

    pub fn connect(room: Arc<crate::ui::Room>, context: egui::Context) -> Result<Self, String> {
        Self::connect_view(room, context, false)
    }

    pub fn connect_room(
        room: Arc<crate::ui::Room>,
        context: egui::Context,
    ) -> Result<Self, String> {
        Self::connect_view(room, context, true)
    }

    pub(super) fn connect_view(
        room: Arc<crate::ui::Room>,
        context: egui::Context,
        room_view: bool,
    ) -> Result<Self, String> {
        let motion = super::motion::Policy::new(&context);
        let worker = if room_view {
            RequestWorker::new_room(room, context)?
        } else {
            RequestWorker::new(room, context)?
        };
        worker.send(br#"{"kind":"snapshot","icons":true}"#)?;
        Ok(Self {
            motion,
            worker: Some(worker),
            room_view: room_view.then(room::State::default),
            pending: Some(Pending::Refresh),
            ..Self::default()
        })
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let now = std::time::Instant::now();
        self.navigation
            .appearance
            .set_update_available(self.updates.available_version());
        if self.snapshot.is_some() {
            if let Some(action) = self.updates.background_action(now, self.pending.is_some()) {
                self.update_request(action);
            }
            if self.pending.is_none() {
                ui.ctx().request_repaint_after(
                    self.updates.next_deadline().saturating_duration_since(now),
                );
            }
        }
        workspace::show(self, ui);
        self.take_mcp_discovery();
        self.show_update_source(ui);
        self.show_updates(ui);
    }

    pub(super) fn send_workspace(&mut self) {
        let Some(snapshot) = self.snapshot.as_ref() else {
            return;
        };
        let Some(worker) = self.worker.as_ref() else {
            self.error = Some("Workspace controller is unavailable".into());
            return;
        };
        let mut snapshot = snapshot.clone();
        snapshot["config"]["roles"] = serde_json::to_value(&self.roles).unwrap();
        match worker.send(&workspace::workspace_save_request(
            &snapshot,
            &self.name,
            &self.repository,
            &self.goal,
            self.workspace_mode,
            &self.profiles,
            &self.coordinator,
        )) {
            Ok(()) => {
                self.error = None;
                self.pending = Some(Pending::Save);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn prepare_plan(&mut self) {
        let Some(revision) = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot["revision"].as_str())
        else {
            self.error = Some("Workspace revision is unavailable".into());
            return;
        };
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(&workspace::prepare_request(revision)));
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(Pending::Prepare);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn start_plan(&mut self, revision: &str, digest: &str) {
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Workspace proposal is unavailable".to_owned())
            .and_then(|snapshot| workspace::start_request(snapshot, revision, digest))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                self.confirming_start = None;
                self.error = None;
                self.workspace_next_refresh = None;
                self.pending = Some(Pending::Start);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn refresh_workspace(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"snapshot","icons":true}"#));
        match result {
            Ok(()) => {
                self.pending = Some(if self.navigation.stop_for_delete() {
                    Pending::StopPlanningRefresh
                } else if self.starting_run.is_some() {
                    Pending::StartRefresh
                } else {
                    Pending::Refresh
                })
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn request_metadata(&mut self, event: workspace::profile_tools::Event) {
        let Some(revision) = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot["revision"].as_str())
        else {
            self.error = Some("Workspace revision is unavailable".into());
            return;
        };
        let packet = workspace::profile_tools::metadata_request(&event, revision);
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(&packet));
        match result {
            Ok(()) => {
                self.pending = Some(Pending::Metadata);
                if let workspace::profile_tools::Event::Quota(agent) = event {
                    self.quota_refresh_last
                        .insert(agent, super::quota_refresh::now());
                }
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn refresh_metadata(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"snapshot"}"#));
        match result {
            Ok(()) => self.pending = Some(Pending::MetadataRefresh),
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn pick_folder(&mut self) {
        let Some(worker) = self.worker.as_ref() else {
            self.error = Some("Workspace controller is unavailable".into());
            return;
        };
        match worker.pick_folder(crate::ui::picker::FolderPurpose::Project) {
            Ok(()) => {
                self.error = None;
                self.pending = Some(Pending::PickFolder);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn pick_executable(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(RequestWorker::pick_executable);
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(Pending::PickExecutable);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn resolve_executable(&mut self, executable: String) {
        let packet = serde_json::json!({"kind":"resolve_executable","executable":executable});
        let result = serde_json::to_vec(&packet)
            .map_err(|error| error.to_string())
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                self.executable_resolution = Some(executable);
                self.pending = Some(Pending::ResolveExecutable);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn send_profile_save(&mut self) {
        let proposed = match self.profile_editor.proposed_profiles(&self.profiles) {
            Ok(profiles) => profiles,
            Err(error) => {
                self.profile_editor
                    .set_save_error(self.navigation.appearance.language(), &error);
                return;
            }
        };
        let renames = self.profile_editor.renames().clone();
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Workspace revision is unavailable".to_owned())
            .and_then(|snapshot| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                    .send(&workspace::profile_tools::profile_save_request(
                        snapshot, &proposed, &renames,
                    ))
            });
        match result {
            Ok(()) => {
                self.profile_saved = false;
                self.error = None;
                self.pending = Some(Pending::ProfileSave);
            }
            Err(error) if self.profile_editor.is_editing() => {
                self.profile_editor
                    .set_save_error(self.navigation.appearance.language(), &error);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub fn render_snapshot(snapshot: &Value, ui: &mut egui::Ui) {
        render::snapshot(snapshot, "en", ui);
    }

    pub fn render_snapshot_in_language(snapshot: &Value, language: &str, ui: &mut egui::Ui) {
        render::snapshot(snapshot, language, ui);
    }
}
