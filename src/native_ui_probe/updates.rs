use eframe::egui::{self, RichText};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Status,
    Check,
    Apply,
}

#[derive(Default)]
pub struct State {
    pub open: bool,
    status: Value,
    next_poll: Option<Instant>,
    error: Option<String>,
}

impl State {
    pub fn status(&self) -> &Value {
        &self.status
    }

    pub fn apply_status(&mut self, status: Value) {
        let active = matches!(
            status["phase"].as_str(),
            Some("checking" | "installing" | "restarting")
        );
        self.status = status;
        self.error = None;
        self.next_poll = active.then(|| Instant::now() + Duration::from_millis(1500));
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.next_poll = None;
    }

    pub fn show(&mut self, language: &str, pending: bool, ui: &mut egui::Ui) -> Option<Action> {
        if !self.open {
            return None;
        }
        let text = |id| crate::ui::messages::in_language(language, id);
        let mut action = None;
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(text("startup.updates"));
                if ui.button(text("startup.close")).clicked() {
                    self.open = false;
                }
            });
            let current = self.status["current"]
                .as_str()
                .unwrap_or(env!("CARGO_PKG_VERSION"));
            let latest = self.status["latest"].as_str().unwrap_or_default();
            let current_label = crate::ui::messages::interpolate(
                text("startup.update_installed_version"),
                &[("value1", current)],
            );
            let latest_label = crate::ui::messages::interpolate(
                text("startup.update_latest_version"),
                &[("value1", latest)],
            );
            ui.label(if latest.is_empty() {
                current_label
            } else {
                format!("{current_label} · {latest_label}")
            });
            let phase = self.status["phase"].as_str().unwrap_or("idle");
            let state_label =
                if let Some(error) = self.error.as_deref().or(self.status["error"].as_str()) {
                    crate::ui::messages::interpolate(
                        text("startup.update_failed"),
                        &[("value1", error)],
                    )
                } else if phase == "checking" {
                    text("startup.update_checking").to_owned()
                } else if matches!(phase, "installing" | "restarting") {
                    text("startup.update_installing").to_owned()
                } else if self.status["newer"] == true && self.status["installable"] != true {
                    text("startup.update_unsupported").to_owned()
                } else if self.status["newer"] == true {
                    text("startup.update_newer").to_owned()
                } else {
                    text("startup.update_up_to_date").to_owned()
                };
            ui.label(state_label);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        !pending,
                        egui::Button::new(text("startup.check_for_updates")),
                    )
                    .clicked()
                {
                    action = Some(Action::Check);
                }
                if self.status["installable"] == true
                    && ui
                        .add_enabled(
                            !pending && phase == "idle",
                            egui::Button::new(text("startup.update_and_restart")),
                        )
                        .clicked()
                {
                    action = Some(Action::Apply);
                }
            });
            let page = self.status["page"].as_str().unwrap_or_default();
            if !page.is_empty()
                && ui
                    .button(text("startup.update_release_notes"))
                    .accessible_name(format!("{} · {page}", text("startup.update_release_notes")))
                    .clicked()
            {
                ui.ctx().copy_text(page.to_owned());
            }
            let notes = self.status["notes"].as_str().unwrap_or_default();
            if !notes.is_empty() {
                egui::ScrollArea::vertical()
                    .id_salt("native-release-notes")
                    .max_height(260.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(notes).text_style(egui::TextStyle::Body),
                            )
                            .selectable(true),
                        );
                    });
            }
        });
        if action.is_none()
            && !pending
            && let Some(next) = self.next_poll
        {
            if next <= Instant::now() {
                action = Some(Action::Status);
            } else {
                ui.ctx().request_repaint_after(next - Instant::now());
            }
        }
        action
    }
}

pub fn request(action: Action) -> Vec<u8> {
    let kind = match action {
        Action::Status => "update_status",
        Action::Check => "check_update",
        Action::Apply => "apply_update",
    };
    serde_json::to_vec(&json!({"kind":kind})).expect("update request is serializable")
}

impl super::Probe {
    pub(super) fn open_updates(&mut self) {
        self.updates.open = true;
        self.update_request(Action::Check);
    }

    pub(super) fn show_updates(&mut self, ui: &mut egui::Ui) {
        let action = self.updates.show(
            self.navigation.appearance.language(),
            self.pending.is_some(),
            ui,
        );
        if let Some(action) = action {
            self.update_request(action);
        }
    }

    fn update_request(&mut self, action: Action) {
        if self.pending.is_some() {
            self.updates.set_error(
                self.navigation
                    .appearance
                    .text("native.update_already_in_progress")
                    .to_owned(),
            );
            return;
        }
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(&request(action)));
        match result {
            Ok(()) => {
                self.pending = Some(super::Pending::Update);
                self.error = None;
            }
            Err(error) => self.updates.set_error(error),
        }
    }
}
