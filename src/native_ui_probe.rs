use eframe::egui;
use serde_json::Value;
use std::sync::Arc;

mod app;
mod pending;
mod worker;
pub use app::{Surface, open_window};
use pending::Pending;
pub use worker::RequestWorker;

pub mod cache;
mod poll;
mod render;
pub mod room;
mod room_actions;
mod room_briefing;
pub mod workspace;
pub use render::evidence as render_evidence;
pub use render::{status_color, status_color_with_motion};
pub use room::{CandidateDecision, candidate_decision_request, refresh_due, room_refresh_interval};
pub use room_actions::{RunAction, ToolAction, run_action_request, tool_action_request};
pub use workspace::{picked_folder, profile_from_template, workspace_save_request};

#[derive(Default)]
pub struct Probe {
    room_view: Option<room::State>,
    workspace_step: usize,
    workspace_next_refresh: Option<std::time::Instant>,
    name: String,
    repository: String,
    goal: String,
    workspace_mode: crate::plan::WorkspaceMode,
    profiles: Vec<Value>,
    profile_editor: workspace::profile_tools::Editor,
    executable_resolution: Option<String>,
    coordinator: String,
    roles: crate::roles::RoleRules,
    worker: Option<RequestWorker>,
    snapshot: Option<Value>,
    error: Option<String>,
    pending: Option<Pending>,
    resume_after_tool_approval: bool,
    confirming_start: Option<(String, String)>,
    task_draft: Option<workspace::TaskDraft>,
    task_saved_notice: bool,
    starting_run: Option<String>,
    cache: cache::State,
    navigation: workspace::navigation::State,
}

impl Probe {
    pub fn connect(room: Arc<crate::ui::Room>, context: egui::Context) -> Result<Self, String> {
        Self::connect_view(room, context, false)
    }

    pub fn connect_room(
        room: Arc<crate::ui::Room>,
        context: egui::Context,
    ) -> Result<Self, String> {
        Self::connect_view(room, context, true)
    }

    fn connect_view(
        room: Arc<crate::ui::Room>,
        context: egui::Context,
        room_view: bool,
    ) -> Result<Self, String> {
        let worker = RequestWorker::new(room, context)?;
        worker.send(br#"{"kind":"snapshot"}"#)?;
        Ok(Self {
            worker: Some(worker),
            room_view: room_view.then(room::State::default),
            pending: room_view.then_some(Pending::Refresh),
            ..Self::default()
        })
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        workspace::show(self, ui);
    }

    fn send_workspace(&mut self) {
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

    fn prepare_plan(&mut self) {
        self.task_saved_notice = false;
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
                self.task_saved_notice = false;
                self.error = None;
                self.workspace_next_refresh = None;
                self.pending = Some(Pending::Start);
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn refresh_workspace(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"snapshot"}"#));
        match result {
            Ok(()) => {
                self.pending = Some(if self.starting_run.is_some() {
                    Pending::StartRefresh
                } else {
                    Pending::Refresh
                })
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn request_metadata(&mut self, event: workspace::profile_tools::Event) {
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
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn refresh_metadata(&mut self) {
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

    fn pick_folder(&mut self) {
        let Some(worker) = self.worker.as_ref() else {
            self.error = Some("Workspace controller is unavailable".into());
            return;
        };
        match worker.pick_folder() {
            Ok(()) => {
                self.error = None;
                self.pending = Some(Pending::PickFolder);
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn pick_executable(&mut self) {
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

    fn resolve_executable(&mut self, executable: String) {
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

    fn send_profile_save(&mut self) {
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
                        snapshot,
                        &self.profiles,
                        &renames,
                    ))
            });
        match result {
            Ok(()) => {
                workspace::profile_tools::rename_references(
                    &mut self.profiles,
                    &mut self.coordinator,
                    &mut self.roles,
                    &renames,
                );
                self.error = None;
                self.pending = Some(Pending::ProfileSave);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub fn render_snapshot(snapshot: &Value, ui: &mut egui::Ui) {
        render::snapshot(snapshot, ui);
    }
}
