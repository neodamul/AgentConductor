use eframe::egui;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

mod request;
mod source;
pub use source::{Action as SourceAction, Editor as SourceEditor};

const CHECK_INTERVAL: Duration = Duration::from_secs(600);
const STATUS_INTERVAL: Duration = Duration::from_millis(1500);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Status,
    Check,
    Apply,
}

pub struct State {
    pub open: bool,
    status: Value,
    next_poll: Option<Instant>,
    next_check: Instant,
    error: Option<String>,
    phase_progress: Option<(String, super::progress::WaitClock)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            open: false,
            status: Value::Null,
            next_poll: None,
            next_check: Instant::now(),
            error: None,
            phase_progress: None,
        }
    }
}

impl State {
    pub fn status(&self) -> &Value {
        &self.status
    }

    pub fn available_version(&self) -> Option<&str> {
        (self.status["newer"] == true)
            .then(|| self.status["latest"].as_str())
            .flatten()
            .filter(|version| !version.is_empty())
    }

    pub fn background_action(&mut self, now: Instant, pending: bool) -> Option<Action> {
        if pending {
            return None;
        }
        if let Some(next) = self.next_poll {
            if next <= now {
                self.next_poll = Some(now + STATUS_INTERVAL);
                return Some(Action::Status);
            }
            return None;
        }
        if self.next_check <= now {
            self.next_check = now + CHECK_INTERVAL;
            return Some(Action::Check);
        }
        None
    }

    pub fn next_deadline(&self) -> Instant {
        self.next_poll
            .map_or(self.next_check, |next| next.min(self.next_check))
    }

    fn requested(&mut self, action: Action) {
        self.requested_at(action, Instant::now());
    }

    fn requested_at(&mut self, action: Action, now: Instant) {
        if action == Action::Check {
            self.next_check = now + CHECK_INTERVAL;
            self.phase_progress = Some(("checking".into(), super::progress::WaitClock::start(now)));
        } else if action == Action::Apply {
            self.phase_progress =
                Some(("installing".into(), super::progress::WaitClock::start(now)));
        }
    }

    pub fn apply_status(&mut self, status: Value) {
        self.apply_status_at(status, Instant::now());
    }

    fn apply_status_at(&mut self, status: Value, now: Instant) {
        let active = matches!(
            status["phase"].as_str(),
            Some("checking" | "installing" | "restarting")
        );
        if let Some(phase) = status["phase"].as_str().filter(|_| active) {
            if let Some((current, clock)) = &mut self.phase_progress {
                if current != phase {
                    *clock = super::progress::WaitClock::start(now);
                } else {
                    clock.observe(now);
                }
                *current = phase.to_owned();
            } else {
                self.phase_progress =
                    Some((phase.to_owned(), super::progress::WaitClock::start(now)));
            }
        } else {
            self.phase_progress = None;
        }
        self.status = status;
        self.error = None;
        self.next_poll = active.then(|| now + STATUS_INTERVAL);
    }

    fn wait_label(&self, language: &str, now: Instant) -> Option<String> {
        self.phase_progress
            .as_ref()
            .map(|(_, clock)| super::progress::label(*clock, language, now))
    }

    pub(super) fn is_waiting(&self) -> bool {
        self.phase_progress.is_some()
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
        let mut open = self.open;
        let mut close_requested = false;
        egui::Window::new(text("startup.updates"))
            .id(egui::Id::NULL.with("native-update-dialog"))
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .default_width(480.0)
            .max_width(720.0)
            .max_height(620.0)
            .vscroll(true)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.heading(text("startup.updates"));
                    if ui.button(text("startup.close")).clicked() {
                        close_requested = true;
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
                if let Some(progress) = self.wait_label(language, Instant::now()) {
                    ui.small(progress);
                }
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
                if page.starts_with("https://github.com/neodamul/AgentConductor/releases/") {
                    ui.hyperlink_to(text("startup.update_release_notes"), page)
                        .accessible_name(format!(
                            "{} · {page}",
                            text("startup.update_release_notes")
                        ));
                }
                let notes = self.status["notes"].as_str().unwrap_or_default();
                if !notes.is_empty() {
                    let mut notes = notes;
                    egui::ScrollArea::vertical()
                        .id_salt("native-release-notes")
                        .max_height(260.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut notes)
                                    .font(egui::TextStyle::Body)
                                    .desired_rows(8)
                                    .desired_width(ui.available_width())
                                    .frame(egui::Frame::NONE),
                            );
                        });
                }
            });
        self.open = open && !close_requested;
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

#[cfg(test)]
#[path = "../../tests/native_ui_probe/update_progress.rs"]
mod progress_tests;
