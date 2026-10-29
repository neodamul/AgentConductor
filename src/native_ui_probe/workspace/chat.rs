use eframe::egui::{self, RichText};
mod launcher;
mod outcome;
mod reset;
mod send;

pub enum Event {
    Send(Vec<crate::chat::Turn>),
    NewConversation,
}

#[derive(Default)]
pub struct State {
    open: bool,
    draft: String,
    turns: Vec<crate::chat::Turn>,
    pending_before: Option<Vec<crate::chat::Turn>>,
    pending_turn: Option<crate::chat::Turn>,
    uncertain_error: Option<String>,
    requesting: bool,
    resetting: bool,
    job: Option<String>,
    error: Option<String>,
}

impl State {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn awaiting(&self) -> bool {
        self.requesting || self.uncertain_error.is_some() || self.resetting || self.job.is_some()
    }

    pub fn is_uncertain(&self) -> bool {
        self.uncertain_error.is_some()
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn reset(&mut self) {
        self.turns.clear();
        self.pending_before = None;
        self.draft.clear();
        self.pending_turn = None;
        self.uncertain_error = None;
        self.requesting = false;
        self.resetting = false;
        self.job = None;
        self.error = None;
    }

    pub fn show(&mut self, language: &str, ui: &mut egui::Ui) -> Option<Event> {
        let mut new_conversation = false;
        ui.horizontal(|ui| {
            ui.heading(text(language, "chat.title"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(text(language, "chat.close")).clicked() {
                    self.open = false;
                }
                if !self.turns.is_empty()
                    && !self.awaiting()
                    && ui.button(text(language, "chat.new")).clicked()
                {
                    new_conversation = true;
                }
            });
        });
        if new_conversation {
            return Some(Event::NewConversation);
        }
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
        self.pending_before = Some(self.turns.clone());
        self.turns.push(turn.clone());
        self.pending_turn = Some(turn);
        self.error = None;
        self.requesting = true;
        Some(Event::Send(self.turns.clone()))
    }

    pub fn started(&mut self, job: String) {
        self.pending_before = None;
        self.pending_turn = None;
        self.uncertain_error = None;
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
        self.turns = crate::chat::recent_history(&self.turns);
        self.job = None;
        self.requesting = false;
        true
    }

    pub fn failed(&mut self, message: String) {
        self.pending_before = None;
        self.job = None;
        self.requesting = false;
        self.uncertain_error = None;
        self.error = Some(message);
    }

    pub fn rejected(&mut self, message: String) {
        self.pending_before = None;
        self.uncertain_error = None;
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
    let revision = snapshot["chat_revision"]
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

pub(in crate::native_ui_probe) fn load_history(
    probe: &mut super::Probe,
    snapshot: &serde_json::Value,
) {
    probe.coordinator_chat.load_snapshot(snapshot);
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}
