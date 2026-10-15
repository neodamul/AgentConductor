use eframe::egui::{self, Color32, CornerRadius, Frame, RichText, Stroke, vec2};
use serde_json::Value;
use std::sync::{Arc, mpsc};

mod render;
pub mod room;
mod workspace;
pub use render::evidence as render_evidence;
pub use render::status_color;
pub use room::{refresh_due, room_refresh_interval};
pub use workspace::{picked_folder, profile_from_template, workspace_save_request};

#[derive(Clone, Copy)]
enum Pending {
    Save,
    Refresh,
    Action,
    Evidence,
    PickFolder,
}

enum WorkerRequest {
    Dispatch(Vec<u8>),
    PickFolder,
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
}

#[derive(Default)]
pub struct Probe {
    room_view: Option<room::State>,
    name: String,
    repository: String,
    goal: String,
    workspace_mode: crate::plan::WorkspaceMode,
    profiles: Vec<Value>,
    coordinator: String,
    worker: Option<RequestWorker>,
    snapshot: Option<Value>,
    error: Option<String>,
    pending: Option<Pending>,
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
        let background = Color32::from_rgb(242, 242, 246);
        ui.painter().rect_filled(ui.max_rect(), 0.0, background);
        ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
        Frame::new()
            .fill(background)
            .inner_margin(24.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("AC")
                            .strong()
                            .color(Color32::WHITE)
                            .background_color(Color32::from_rgb(10, 102, 216)),
                    );
                    ui.heading("AgentConductor");
                    ui.label(
                        RichText::new("NATIVE UI PROBE")
                            .small()
                            .color(Color32::from_rgb(96, 96, 103)),
                    );
                });
                if let Some(room_view) = self.room_view.as_mut() {
                    if let Some(event) = room::show(
                        room_view,
                        self.snapshot.as_ref(),
                        self.pending.is_some(),
                        self.error.as_deref(),
                        ui,
                    ) {
                        match event {
                            room::Event::Refresh => self.refresh_room(),
                            room::Event::Stop(reason) => self.stop_room(reason),
                            room::Event::Evidence => self.load_evidence(),
                        }
                    }
                    return;
                }
                ui.label("Plan  /  Task  /  Verify  /  Complete");
                if let Some(snapshot) = &self.snapshot {
                    Self::render_snapshot(snapshot, ui);
                } else if let Some(error) = &self.error {
                    ui.colored_label(Color32::DARK_RED, error);
                } else {
                    ui.label("Connecting to the Rust workspace controller…");
                }
                let can_save = self.snapshot.is_some() && self.pending.is_none();
                Frame::new()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.0, Color32::from_rgb(220, 220, 226)))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("WORKSPACE").small().strong());
                        let name_label = ui.label("Workspace name");
                        let name_editor = ui
                            .add_enabled_ui(self.pending.is_none(), |ui| {
                                ui.add_sized(
                                    vec2(ui.available_width(), 32.0),
                                    egui::TextEdit::singleline(&mut self.name),
                                )
                            })
                            .inner;
                        name_editor
                            .labelled_by(name_label.id)
                            .accessible_name("Workspace name");
                        ui.horizontal(|ui| {
                            let repository_label = ui.label("Repository path");
                            let repository_editor = ui
                                .add_enabled_ui(self.pending.is_none(), |ui| {
                                    ui.add_sized(
                                        vec2(ui.available_width().min(420.0), 32.0),
                                        egui::TextEdit::singleline(&mut self.repository),
                                    )
                                })
                                .inner;
                            repository_editor
                                .labelled_by(repository_label.id)
                                .accessible_name("Repository path");
                            let browse = ui
                                .add_enabled(self.pending.is_none(), egui::Button::new("Browse…"));
                            if browse
                                .accessible_name("Browse for project folder")
                                .clicked()
                            {
                                self.pick_folder();
                            }
                        });
                        let label = ui.label("Goal");
                        let editor = ui
                            .add_enabled_ui(self.pending.is_none(), |ui| {
                                ui.add_sized(
                                    vec2(ui.available_width(), 32.0),
                                    egui::TextEdit::singleline(&mut self.goal),
                                )
                            })
                            .inner;
                        editor.labelled_by(label.id).accessible_name("Goal");
                        ui.label("Execution isolation");
                        ui.add_enabled_ui(self.pending.is_none(), |ui| {
                            ui.horizontal(|ui| {
                                ui.radio_value(
                                    &mut self.workspace_mode,
                                    crate::plan::WorkspaceMode::Shared,
                                    "Shared folder",
                                );
                                ui.radio_value(
                                    &mut self.workspace_mode,
                                    crate::plan::WorkspaceMode::Worktree,
                                    "Git worktree",
                                );
                            });
                        });
                        if ui
                            .add_enabled(can_save, egui::Button::new("Save workspace"))
                            .clicked()
                        {
                            self.send_workspace();
                        }
                    });
                if let Some(snapshot) = &self.snapshot {
                    Self::show_team(
                        &mut self.profiles,
                        &mut self.coordinator,
                        self.pending.is_none(),
                        snapshot,
                        ui,
                    );
                }
                if self.pending.is_some() {
                    ui.spinner();
                    ui.label(if matches!(self.pending, Some(Pending::PickFolder)) {
                        "Choosing project folder…"
                    } else {
                        "Saving workspace…"
                    });
                }
                if let Some(error) = &self.error {
                    ui.colored_label(Color32::DARK_RED, error);
                }
                ui.label("Korean input: 한국어 입력 확인");
            });
    }

    fn show_team(
        profiles: &mut Vec<Value>,
        coordinator: &mut String,
        editable: bool,
        snapshot: &Value,
        ui: &mut egui::Ui,
    ) {
        Frame::new()
            .fill(Color32::WHITE)
            .stroke(Stroke::new(1.0, Color32::from_rgb(220, 220, 226)))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.label(RichText::new("TEAM").small().strong());
                let mut selected = None;
                ui.add_enabled_ui(editable && profiles.len() < 16, |ui| {
                    ui.menu_button("Add agent", |ui| {
                        if let Some(templates) = snapshot["templates"].as_array() {
                            for template in templates {
                                let id = template["id"].as_str().unwrap_or("agent");
                                let parser = template["parser"].as_str().unwrap_or("custom");
                                if ui.button(format!("{id} · {parser}")).clicked() {
                                    selected = Some(template.clone());
                                    ui.close();
                                }
                            }
                        }
                    });
                });
                if let Some(template) = selected
                    && let Some(profile) = profile_from_template(&template, profiles)
                {
                    let id = profile["id"].as_str().unwrap_or_default().to_owned();
                    if coordinator.is_empty() {
                        *coordinator = id;
                    }
                    profiles.push(profile);
                }
                for profile in profiles.iter() {
                    let id = profile["id"].as_str().unwrap_or("unknown").to_owned();
                    let parser = profile["parser"].as_str().unwrap_or("custom");
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{id} · {parser}")).strong());
                        ui.add_enabled_ui(editable, |ui| {
                            ui.radio_value(coordinator, id.clone(), format!("Coordinator: {id}"));
                        });
                    });
                }
            });
    }

    fn send_workspace(&mut self) {
        let Some(snapshot) = self.snapshot.as_ref() else {
            return;
        };
        let Some(worker) = self.worker.as_ref() else {
            self.error = Some("Workspace controller is unavailable".into());
            return;
        };
        match worker.send(&workspace::workspace_save_request(
            snapshot,
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

    fn poll(&mut self) {
        let Some(worker) = &self.worker else { return };
        match worker.replies.try_recv() {
            Ok(Ok(snapshot)) => match self.pending {
                Some(Pending::PickFolder) => {
                    self.repository = picked_folder(&self.repository, &snapshot);
                    self.pending = None;
                    self.error = None;
                }
                Some(Pending::Save) if snapshot["saved"] == true => {
                    if let Err(error) = worker.send(br#"{"kind":"snapshot"}"#) {
                        self.pending = None;
                        self.error = Some(error);
                    } else {
                        self.pending = Some(Pending::Refresh);
                    }
                }
                Some(Pending::Save) => {
                    self.pending = None;
                    self.error = Some("Workspace goal was not saved".into());
                }
                Some(Pending::Action) => match room::action_reply(worker, &snapshot) {
                    Ok(message) => {
                        if let Some(room_view) = self.room_view.as_mut() {
                            room_view.notice = message;
                        }
                        self.pending = Some(Pending::Refresh);
                    }
                    Err(error) => {
                        self.pending = None;
                        self.error = Some(error);
                    }
                },
                Some(Pending::Evidence) => {
                    let current = self.snapshot.as_ref();
                    match self
                        .room_view
                        .as_mut()
                        .ok_or_else(|| "Agent Room is unavailable".to_owned())
                        .and_then(|room_view| room::evidence_reply(room_view, current, snapshot))
                    {
                        Ok(()) => {
                            self.pending = None;
                            self.error = None;
                        }
                        Err(error) => {
                            self.pending = None;
                            self.error = Some(error);
                        }
                    }
                }
                Some(Pending::Refresh) | None => {
                    if let Some(room_view) = self.room_view.as_mut() {
                        if room_view
                            .evidence
                            .as_ref()
                            .is_some_and(|evidence| evidence["revision"] != snapshot["revision"])
                        {
                            room_view.evidence = None;
                            room_view.diff.clear();
                        }
                    } else {
                        self.goal = snapshot["config"]["goal"]
                            .as_str()
                            .unwrap_or_default()
                            .to_owned();
                        self.name = snapshot["config"]["name"]
                            .as_str()
                            .unwrap_or_default()
                            .to_owned();
                        self.repository = snapshot["config"]["repository"]
                            .as_str()
                            .unwrap_or_default()
                            .to_owned();
                        self.workspace_mode =
                            serde_json::from_value(snapshot["config"]["workspace_mode"].clone())
                                .unwrap_or_default();
                        self.profiles = snapshot["config"]["profiles"]
                            .as_array()
                            .cloned()
                            .unwrap_or_default();
                        self.coordinator = snapshot["config"]["coordinator"]
                            .as_str()
                            .unwrap_or_default()
                            .to_owned();
                    }
                    self.snapshot = Some(snapshot);
                    self.pending = None;
                    self.error = None;
                }
            },
            Ok(Err(error)) => {
                self.pending = None;
                self.error = Some(error);
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(error) => {
                self.pending = None;
                self.error = Some(error.to_string());
            }
        }
    }

    pub fn render_snapshot(snapshot: &Value, ui: &mut egui::Ui) {
        render::snapshot(snapshot, ui);
    }
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
        self.show(ui);
    }
}
