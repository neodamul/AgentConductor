use eframe::egui::{self, TextEdit};
use serde_json::Value;

#[derive(Debug, Eq, PartialEq)]
pub enum Event {
    Save(Vec<u8>),
    Test(Vec<u8>),
}

#[derive(Default)]
pub struct State {
    open: bool,
    settings: crate::messenger::Settings,
    url: String,
    token: String,
    relay_url: String,
    relay_secret: String,
    approvers: String,
    has_url: bool,
    has_token: bool,
    has_relay: bool,
    has_relay_secret: bool,
    notice: Option<String>,
}

impl State {
    pub fn open(&mut self, snapshot: &Value) {
        self.settings =
            serde_json::from_value(snapshot["config"]["messenger"].clone()).unwrap_or_default();
        self.approvers = self.settings.approvers.join(", ");
        self.url.clear();
        self.token.clear();
        self.relay_url.clear();
        self.relay_secret.clear();
        self.notice = None;
        self.open = true;
    }

    pub(in crate::native_ui_probe) fn apply_status(&mut self, status: &Value) {
        self.settings = serde_json::from_value(status["settings"].clone()).unwrap_or_default();
        self.has_url = status["has_url"] == true;
        self.has_token = status["has_token"] == true;
        self.has_relay = status["has_relay"] == true;
        self.has_relay_secret = status["has_relay_secret"] == true;
    }

    pub(in crate::native_ui_probe) fn saved(&mut self) {
        self.open = false;
    }

    pub(in crate::native_ui_probe) fn test_sent(&mut self, language: &str) {
        self.notice = Some(text(language, "startup.test_message_sent").into());
    }

    pub fn show(
        &mut self,
        snapshot: &Value,
        language: &str,
        can_edit: bool,
        ui: &mut egui::Ui,
    ) -> Option<Event> {
        if !self.open {
            return None;
        }
        let mut event = None;
        let mut open = self.open;
        egui::Window::new(text(language, "startup.messenger_notifications"))
            .open(&mut open)
            .default_width(600.0)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.add_enabled_ui(can_edit, |ui| {
                    ui.checkbox(
                        &mut self.settings.enabled,
                        text(language, "startup.send_notifications_for_this_workspace"),
                    );
                    ui.label(text(language, "startup.messenger_9a523e"));
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.settings.kind,
                            crate::messenger::Kind::Slack,
                            text(language, "startup.slack_incoming_webhook"),
                        );
                        ui.selectable_value(
                            &mut self.settings.kind,
                            crate::messenger::Kind::Webhook,
                            text(language, "startup.company_messenger_webhook"),
                        );
                    });
                    let url_label = ui.label(text(language, "startup.webhook_url"));
                    ui.add(
                        TextEdit::singleline(&mut self.url)
                            .password(true)
                            .desired_width(ui.available_width()),
                    )
                    .labelled_by(url_label.id)
                    .accessible_name(text(language, "startup.webhook_url"));
                    if self.has_url {
                        ui.label(text(
                            language,
                            "startup.a_webhook_url_is_saved_on_this_computer",
                        ));
                    }
                    ui.label(text(language, "startup.notify_when"));
                    ui.horizontal_wrapped(|ui| {
                        for state in crate::messenger::EVENTS {
                            let mut selected =
                                self.settings.events.iter().any(|event| event == state);
                            if ui
                                .checkbox(&mut selected, event_label(state, language))
                                .changed()
                            {
                                self.settings.events.retain(|event| event != state);
                                if selected {
                                    self.settings.events.push(state.into());
                                }
                            }
                        }
                    });
                    if self.settings.kind == crate::messenger::Kind::Webhook {
                        let label = ui.label(text(language, "startup.message_body_json"));
                        ui.add(
                            TextEdit::multiline(
                                self.settings.template.get_or_insert_with(String::new),
                            )
                            .code_editor()
                            .desired_rows(3)
                            .desired_width(ui.available_width()),
                        )
                        .labelled_by(label.id)
                        .accessible_name(text(language, "startup.message_body_json"));
                    } else {
                        field(
                            ui,
                            language,
                            "startup.slack_channel_id",
                            self.settings.slack_channel.get_or_insert_default(),
                            false,
                        );
                        field(
                            ui,
                            language,
                            "startup.slack_bot_token",
                            &mut self.token,
                            true,
                        );
                        saved(
                            ui,
                            language,
                            self.has_token,
                            "startup.slack_bot_token_saved",
                        );
                    }
                    ui.checkbox(
                        &mut self.settings.include_goal,
                        text(language, "startup.include_the_goal_text_in_messages"),
                    );
                    ui.checkbox(
                        &mut self.settings.decisions,
                        text(language, "startup.let_listed_approvers_reply_allow_or_deny"),
                    );
                    if self.settings.kind == crate::messenger::Kind::Webhook {
                        field(ui, language, "startup.relay_url", &mut self.relay_url, true);
                        field(
                            ui,
                            language,
                            "startup.relay_secret",
                            &mut self.relay_secret,
                            true,
                        );
                        saved(ui, language, self.has_relay, "startup.relay_url_saved");
                        saved(
                            ui,
                            language,
                            self.has_relay_secret,
                            "startup.relay_secret_saved",
                        );
                    }
                    field(
                        ui,
                        language,
                        "startup.approvers_comma_separated_user_ids",
                        &mut self.approvers,
                        false,
                    );
                    self.settings.approvers = self
                        .approvers
                        .split(',')
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_owned)
                        .collect();
                    if let Some(notice) = &self.notice {
                        ui.label(notice);
                    }
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                !self.url.trim().is_empty() || self.has_url,
                                egui::Button::new(text(language, "startup.send_test_message")),
                            )
                            .clicked()
                            && let Ok(packet) = test_request(&self.settings, optional(&self.url))
                        {
                            event = Some(Event::Test(packet));
                        }
                        if ui
                            .add_enabled(
                                can_edit && snapshot["revision"].is_string(),
                                egui::Button::new(text(language, "startup.save")),
                            )
                            .clicked()
                            && let Ok(packet) =
                                save_request(snapshot, &self.settings, self.secrets())
                        {
                            event = Some(Event::Save(packet));
                        }
                    });
                });
            });
        self.open = open;
        event
    }

    fn secrets(&self) -> [Option<String>; 4] {
        [
            optional(&self.url),
            optional(&self.token),
            optional(&self.relay_url),
            optional(&self.relay_secret),
        ]
    }
}

fn field(ui: &mut egui::Ui, language: &str, id: &'static str, value: &mut String, secret: bool) {
    let label = ui.label(text(language, id));
    ui.add(
        TextEdit::singleline(value)
            .password(secret)
            .desired_width(ui.available_width()),
    )
    .labelled_by(label.id)
    .accessible_name(text(language, id));
}

fn saved(ui: &mut egui::Ui, language: &str, present: bool, id: &'static str) {
    if present {
        ui.label(text(language, id));
    }
}

fn optional(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then(|| value.trim().to_owned())
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

fn event_label(state: &str, language: &str) -> &'static str {
    text(
        language,
        match state {
            "plan_ready" => "startup.plan_ready",
            "waiting_approval" => "startup.candidate_ready_for_review",
            "waiting_tool_approval" => "startup.tool_request_needs_a_decision",
            "waiting_availability" => "startup.waiting_for_account_availability",
            "completed" => "startup.run_completed",
            "failed" => "startup.run_failed",
            "stopped" => "startup.stop",
            "blocked" => "startup.blocked",
            _ => "startup.notify_when",
        },
    )
}

impl super::super::Probe {
    pub(super) fn open_messenger(&mut self) {
        let Some(snapshot) = self.snapshot.as_ref() else {
            return;
        };
        self.messenger_editor.open(snapshot);
        match self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"messenger"}"#))
        {
            Ok(()) => {
                self.pending = Some(super::super::Pending::MessengerLoad);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn send_messenger(&mut self, event: Event) {
        let (packet, pending) = match event {
            Event::Save(packet) => (packet, super::super::Pending::MessengerSave),
            Event::Test(packet) => (packet, super::super::Pending::MessengerTest),
        };
        match self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(&packet))
        {
            Ok(()) => {
                self.pending = Some(pending);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }
}

pub fn test_request(
    settings: &crate::messenger::Settings,
    url: Option<String>,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&serde_json::json!({"kind":"test_messenger","settings":settings,"url":url}))
        .map_err(|error| error.to_string())
}

pub fn save_request(
    snapshot: &Value,
    settings: &crate::messenger::Settings,
    secrets: [Option<String>; 4],
) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .ok_or("workspace revision is unavailable")?;
    let [url, token, relay_url, relay_secret] = secrets;
    serde_json::to_vec(&serde_json::json!({
        "kind":"save_messenger","revision":revision,"settings":settings,
        "url":url,"token":token,"relay_url":relay_url,"relay_secret":relay_secret
    }))
    .map_err(|error| error.to_string())
}
