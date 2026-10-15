use eframe::egui::{self, Color32, CornerRadius, Frame, RichText, Stroke, vec2};
use serde_json::Value;
use std::sync::{Arc, mpsc};

#[derive(Clone, Copy)]
enum Pending {
    Save,
    Refresh,
    PickFolder,
}

enum WorkerRequest {
    Dispatch(Vec<u8>),
    PickFolder,
}

pub fn workspace_save_request(
    snapshot: &Value,
    name: &str,
    repository: &str,
    goal: &str,
) -> Vec<u8> {
    let mut config = snapshot["config"].clone();
    config["name"] = name.into();
    config["repository"] = repository.into();
    config["goal"] = goal.into();
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
    name: String,
    repository: String,
    goal: String,
    worker: Option<RequestWorker>,
    snapshot: Option<Value>,
    error: Option<String>,
    pending: Option<Pending>,
}

impl Probe {
    pub fn connect(room: Arc<crate::ui::Room>, context: egui::Context) -> Result<Self, String> {
        let worker = RequestWorker::new(room, context)?;
        worker.send(br#"{"kind":"snapshot"}"#)?;
        Ok(Self {
            worker: Some(worker),
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
                        if ui
                            .add_enabled(can_save, egui::Button::new("Save workspace"))
                            .clicked()
                        {
                            self.send_workspace();
                        }
                    });
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

    fn send_workspace(&mut self) {
        let Some(snapshot) = self.snapshot.as_ref() else {
            return;
        };
        let Some(worker) = self.worker.as_ref() else {
            self.error = Some("Workspace controller is unavailable".into());
            return;
        };
        match worker.send(&workspace_save_request(
            snapshot,
            &self.name,
            &self.repository,
            &self.goal,
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
                Some(Pending::Refresh) | None => {
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
        if let Some(root) = snapshot["root"].as_str() {
            ui.label(root);
        }
        if let Some(goal) = snapshot["config"]["goal"]
            .as_str()
            .filter(|goal| !goal.is_empty())
        {
            ui.label(RichText::new(goal).strong());
        }
        ui.label(RichText::new("REGISTERED AGENTS").small().strong());
        match snapshot["config"]["profiles"].as_array() {
            Some(profiles) if !profiles.is_empty() => {
                for profile in profiles {
                    let id = profile["id"].as_str().unwrap_or("unknown");
                    let parser = profile["parser"].as_str().unwrap_or("custom");
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(id).strong());
                        ui.label(parser);
                    });
                }
            }
            _ => {
                ui.label("No agents registered");
            }
        }
    }
}

impl eframe::App for Probe {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll();
        ui.ctx().set_visuals(egui::Visuals::light());
        self.show(ui);
    }
}
