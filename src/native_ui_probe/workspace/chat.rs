use eframe::egui::{self, RichText};

pub enum Event {
    Send(Vec<crate::chat::Turn>),
}

#[derive(Default)]
pub struct State {
    open: bool,
    draft: String,
    turns: Vec<crate::chat::Turn>,
    pending_turn: Option<crate::chat::Turn>,
    requesting: bool,
    job: Option<String>,
    error: Option<String>,
}

impl State {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn awaiting(&self) -> bool {
        self.requesting || self.job.is_some()
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn show_launcher(&mut self, language: &str, ui: &mut egui::Ui) {
        if ui.button(text(language, "chat.open")).clicked() {
            self.open = true;
        }
    }

    pub fn show(&mut self, language: &str, ui: &mut egui::Ui) -> Option<Event> {
        ui.horizontal(|ui| {
            ui.heading(text(language, "chat.title"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(text(language, "chat.close")).clicked() {
                    self.open = false;
                }
            });
        });
        ui.separator();
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for turn in &self.turns {
                    ui.label(
                        RichText::new(if turn.role == "operator" {
                            text(language, "chat.you")
                        } else {
                            text(language, "chat.title")
                        })
                        .strong(),
                    );
                    ui.label(&turn.text);
                    ui.add_space(8.0);
                }
                if self.awaiting() {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(text(language, "chat.thinking"));
                    });
                }
                if let Some(error) = &self.error {
                    ui.colored_label(ui.visuals().error_fg_color, error);
                }
            });
        ui.separator();
        ui.add_enabled_ui(!self.awaiting(), |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.draft)
                    .desired_rows(3)
                    .hint_text(text(language, "chat.placeholder")),
            );
        });
        let send = ui
            .add_enabled(
                !self.awaiting() && !self.draft.trim().is_empty(),
                egui::Button::new(text(language, "chat.send")),
            )
            .clicked();
        if !send {
            return None;
        }
        let turn = crate::chat::Turn {
            role: "operator".into(),
            text: std::mem::take(&mut self.draft),
        };
        self.turns.push(turn.clone());
        self.pending_turn = Some(turn);
        self.error = None;
        self.requesting = true;
        Some(Event::Send(self.turns.clone()))
    }

    pub fn started(&mut self, job: String) {
        self.pending_turn = None;
        self.requesting = false;
        self.job = Some(job);
        self.error = None;
    }

    pub fn completed(&mut self, job: &str, answer: String) -> bool {
        if self.job.as_deref() != Some(job) || answer.trim().is_empty() {
            return false;
        }
        self.turns.push(crate::chat::Turn {
            role: "assistant".into(),
            text: answer,
        });
        self.job = None;
        self.requesting = false;
        true
    }

    pub fn failed(&mut self, message: String) {
        self.job = None;
        self.requesting = false;
        self.error = Some(message);
    }

    pub fn rejected(&mut self, message: String) {
        if let Some(turn) = self.pending_turn.take() {
            if self
                .turns
                .last()
                .is_some_and(|last| last.role == turn.role && last.text == turn.text)
            {
                self.turns.pop();
            }
            self.draft = turn.text;
        }
        self.failed(message);
    }
}

pub fn request(
    snapshot: &serde_json::Value,
    turns: &[crate::chat::Turn],
) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Workspace revision is unavailable".to_owned())?;
    let language = if snapshot["language"] == "ko" {
        "Korean"
    } else {
        "English"
    };
    crate::chat::build_prompt(turns, language)?;
    let packet = serde_json::to_vec(&serde_json::json!({
        "kind":"chat", "revision":revision, "turns":turns
    }))
    .map_err(|error| error.to_string())?;
    if packet.len() > 128 * 1024 {
        return Err(crate::ui::messages::text("native.native_request_exceeds_128_kib").into());
    }
    Ok(packet)
}

impl super::super::Probe {
    pub(super) fn send_coordinator_chat(&mut self, turns: Vec<crate::chat::Turn>) {
        let packet = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Workspace revision is unavailable".to_owned())
            .and_then(|snapshot| request(snapshot, &turns));
        let result = packet.and_then(|packet| {
            self.worker
                .as_ref()
                .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                .send(&packet)
        });
        match result {
            Ok(()) => {
                self.pending = Some(super::super::Pending::ChatStart);
                self.error = None;
            }
            Err(error) => {
                self.coordinator_chat.rejected(error);
                self.workspace_next_refresh = None;
            }
        }
    }

    pub(in crate::native_ui_probe) fn refresh_coordinator_chat(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"snapshot","chat_result":true}"#));
        match result {
            Ok(()) => self.pending = Some(super::super::Pending::ChatRefresh),
            Err(error) => {
                self.coordinator_chat.failed(error);
                self.workspace_next_refresh = None;
            }
        }
    }

    pub(in crate::native_ui_probe) fn chat_started(&mut self, snapshot: serde_json::Value) {
        if snapshot["accepted"] == true
            && let Some(job) = snapshot["id"].as_str()
        {
            self.coordinator_chat.started(job.to_owned());
            self.workspace_next_refresh =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(500));
            self.pending = None;
        } else {
            self.coordinator_chat
                .rejected("Coordinator chat job was not accepted".into());
            self.pending = None;
        }
    }

    pub(in crate::native_ui_probe) fn chat_refreshed(&mut self, snapshot: serde_json::Value) {
        let Some(job) = self.coordinator_chat.job.clone() else {
            self.pending = None;
            return;
        };
        if snapshot["job"]["id"] != job || snapshot["job"]["kind"] != "chat" {
            self.coordinator_chat
                .failed("Coordinator chat job no longer matches the request".into());
        } else if snapshot["job"]["state"] == "running" {
            self.snapshot = Some(snapshot);
            self.workspace_next_refresh =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(500));
        } else if snapshot["job"]["state"] == "completed" {
            let answer = snapshot["chat"]["answer"].as_str().unwrap_or_default();
            if !self.coordinator_chat.completed(&job, answer.to_owned()) {
                self.coordinator_chat
                    .failed("Coordinator chat returned no answer".into());
            }
            self.snapshot = Some(snapshot);
            self.workspace_next_refresh = None;
        } else {
            let error = snapshot["job"]["error"]
                .as_str()
                .filter(|text| !text.is_empty())
                .unwrap_or("Coordinator could not answer");
            self.coordinator_chat.failed(error.to_owned());
            self.snapshot = Some(snapshot);
            self.workspace_next_refresh = None;
        }
        self.pending = None;
        self.error = None;
    }
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}
