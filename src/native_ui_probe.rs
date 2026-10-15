use eframe::egui::{self, Color32, CornerRadius, Frame, RichText, Stroke, vec2};
use serde_json::Value;
use std::sync::{Arc, mpsc};

pub struct RequestWorker {
    requests: mpsc::SyncSender<Vec<u8>>,
    pub replies: mpsc::Receiver<Result<Value, String>>,
}

impl RequestWorker {
    pub fn new(room: Arc<crate::ui::Room>, context: egui::Context) -> Result<Self, String> {
        let (requests, queue) = mpsc::sync_channel::<Vec<u8>>(1);
        let (reply_tx, replies) = mpsc::channel();
        std::thread::Builder::new()
            .name("native-ui-requests".into())
            .spawn(move || {
                while let Ok(packet) = queue.recv() {
                    let _ = reply_tx.send(room.dispatch(&packet));
                    context.request_repaint();
                }
            })
            .map_err(|error| error.to_string())?;
        Ok(Self { requests, replies })
    }

    pub fn send(&self, packet: &[u8]) -> Result<(), String> {
        self.requests
            .try_send(packet.to_vec())
            .map_err(|error| error.to_string())
    }
}

#[derive(Default)]
pub struct Probe {
    goal: String,
    running: bool,
    worker: Option<RequestWorker>,
    snapshot: Option<Value>,
    error: Option<String>,
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
                Frame::new()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.0, Color32::from_rgb(220, 220, 226)))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("GOAL").small().strong());
                        let label = ui.label("Goal");
                        ui.add_sized(
                            vec2(ui.available_width(), 32.0),
                            egui::TextEdit::singleline(&mut self.goal),
                        )
                        .labelled_by(label.id)
                        .accessible_name("Goal");
                    });
                ui.label(RichText::new("PROBE-ONLY INPUT").small().strong());
                ui.checkbox(&mut self.running, "Probe accessibility interaction");
                ui.label("Korean input: 한국어 입력 확인");
            });
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
        if let Some(worker) = &self.worker {
            match worker.replies.try_recv() {
                Ok(Ok(snapshot)) => self.snapshot = Some(snapshot),
                Ok(Err(error)) => self.error = Some(error),
                Err(mpsc::TryRecvError::Empty) => {}
                Err(error) => self.error = Some(error.to_string()),
            }
        }
        ui.ctx().set_visuals(egui::Visuals::light());
        self.show(ui);
    }
}
