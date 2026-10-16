use eframe::egui;
use serde_json::Value;
use std::sync::{Arc, mpsc};

mod poll;
mod render;
pub mod room;
mod room_actions;
mod room_briefing;
pub mod workspace;
pub use render::evidence as render_evidence;
pub use render::status_color;
pub use room::{CandidateDecision, candidate_decision_request, refresh_due, room_refresh_interval};
pub use room_actions::{RunAction, ToolAction, run_action_request, tool_action_request};
pub use workspace::{picked_folder, profile_from_template, workspace_save_request};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Surface {
    Workspace,
    AgentRoom,
}

#[derive(Clone, Copy)]
enum Pending {
    Save,
    Refresh,
    Action,
    ToolDecision { resume_after: bool },
    Briefing,
    Start,
    Evidence,
    Prepare,
    Metadata,
    MetadataRefresh,
    PickFolder,
    PickExecutable,
    ProfileSave,
    ProfileRefresh,
    ResolveExecutable,
    Edit,
}

enum WorkerRequest {
    Dispatch(Vec<u8>),
    PickFolder,
    PickExecutable,
}

pub struct RequestWorker {
    requests: mpsc::SyncSender<WorkerRequest>,
    pub replies: mpsc::Receiver<Result<Value, String>>,
}

impl RequestWorker {
    pub fn new(room: Arc<crate::ui::Room>, context: egui::Context) -> Result<Self, String> {
        let (requests, queue) = mpsc::sync_channel::<WorkerRequest>(1);
        let (reply_tx, replies) = mpsc::channel();
        std::thread::Builder::new()
            .name("native-ui-requests".into())
            .spawn(move || {
                while let Ok(request) = queue.recv() {
                    let result = match request {
                        WorkerRequest::Dispatch(packet) => room.dispatch(&packet),
                        WorkerRequest::PickFolder => {
                            crate::ui::picker::pick_folder().and_then(|path| {
                                let path = path
                                    .map(|path| {
                                        path.into_os_string().into_string().map_err(|_| {
                                            crate::ui::messages::text(
                                                "native.non_utf_8_folder_path",
                                            )
                                            .to_owned()
                                        })
                                    })
                                    .transpose()?;
                                Ok(serde_json::json!({"picked_folder": path}))
                            })
                        }
                        WorkerRequest::PickExecutable => crate::ui::picker::pick_executable()
                            .and_then(|path| {
                                let path = path
                                    .map(|path| {
                                        path.into_os_string()
                                            .into_string()
                                            .map_err(|_| "Executable path is not UTF-8".to_owned())
                                    })
                                    .transpose()?;
                                Ok(serde_json::json!({"picked_executable":path}))
                            }),
                    };
                    let _ = reply_tx.send(result);
                    context.request_repaint();
                }
            })
            .map_err(|error| error.to_string())?;
        Ok(Self { requests, replies })
    }

    pub fn send(&self, packet: &[u8]) -> Result<(), String> {
        self.requests
            .try_send(WorkerRequest::Dispatch(packet.to_vec()))
            .map_err(|error| error.to_string())
    }

    pub fn pick_folder(&self) -> Result<(), String> {
        self.requests
            .try_send(WorkerRequest::PickFolder)
            .map_err(|error| error.to_string())
    }

    pub fn pick_executable(&self) -> Result<(), String> {
        self.requests
            .try_send(WorkerRequest::PickExecutable)
            .map_err(|error| error.to_string())
    }
}

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
}

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
                self.pending = Some(Pending::Start);
            }
            Err(error) => self.error = Some(error),
        }
    }

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
            .and_then(|snapshot| workspace::task_edit_request(snapshot, &tasks, mode, digest))
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

    fn refresh_workspace(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"snapshot"}"#));
        match result {
            Ok(()) => self.pending = Some(Pending::Refresh),
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

pub fn open_window(room: Arc<crate::ui::Room>, surface: Surface) -> Result<(), String> {
    let room_view = surface == Surface::AgentRoom;
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([720.0, 480.0])
            .with_title(if room_view {
                "AgentConductor — Agent Room"
            } else {
                "AgentConductor"
            }),
        ..Default::default()
    };
    eframe::run_native(
        "AgentConductor",
        options,
        Box::new(move |context| {
            Probe::connect_surface(room, context.egui_ctx.clone(), surface)
                .map(|probe| Box::new(probe) as Box<dyn eframe::App>)
                .map_err(|error| Box::new(std::io::Error::other(error)) as _)
        }),
    )
    .map_err(|error| error.to_string())
}

impl eframe::App for Probe {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll();
        ui.ctx().set_visuals(egui::Visuals::light());
        if self.room_view.as_mut().is_some_and(|room_view| {
            room::refresh_due(
                ui.ctx(),
                self.snapshot.as_ref(),
                self.pending.is_some(),
                &mut room_view.next_refresh,
            )
        }) {
            self.refresh_room();
        }
        if self.room_view.is_none()
            && self.workspace_step == 2
            && self
                .snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot["job"]["state"] == "running")
            && room::refresh_due(
                ui.ctx(),
                self.snapshot.as_ref(),
                self.pending.is_some(),
                &mut self.workspace_next_refresh,
            )
        {
            self.refresh_workspace();
        }
        if self.room_view.is_none()
            && self.workspace_step == 1
            && self
                .snapshot
                .as_ref()
                .is_some_and(workspace::profile_tools::metadata_running)
            && room::refresh_due(
                ui.ctx(),
                self.snapshot.as_ref(),
                self.pending.is_some(),
                &mut self.workspace_next_refresh,
            )
        {
            self.refresh_metadata();
        }
        self.show(ui);
    }
}
