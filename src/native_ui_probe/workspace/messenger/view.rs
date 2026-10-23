use super::{Event, State};
use eframe::egui::{self, TextEdit};
use serde_json::Value;

impl State {
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
        let mut close_requested = false;
        let mut open = self.open;
        egui::Window::new(super::text(language, "startup.messenger_notifications"))
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .default_width(720.0)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.add_enabled_ui(can_edit, |ui| {
                    ui.checkbox(
                        &mut self.settings.enabled,
                        super::text(language, "startup.send_notifications_for_this_workspace"),
                    );
                    ui.label(super::text(language, "startup.messenger_9a523e"));
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.settings.kind,
                            crate::messenger::Kind::Slack,
                            super::text(language, "startup.slack_incoming_webhook"),
                        );
                        ui.selectable_value(
                            &mut self.settings.kind,
                            crate::messenger::Kind::Webhook,
                            super::text(language, "startup.company_messenger_webhook"),
                        );
                    });
                    let url_label = ui.label(super::text(language, "startup.webhook_url"));
                    ui.add(
                        TextEdit::singleline(&mut self.url)
                            .password(true)
                            .desired_width(ui.available_width()),
                    )
                    .labelled_by(url_label.id)
                    .accessible_name(super::text(language, "startup.webhook_url"));
                    if self.has_url {
                        ui.label(super::text(
                            language,
                            "startup.a_webhook_url_is_saved_on_this_computer",
                        ));
                    }
                    ui.label(super::text(language, "startup.notify_when"));
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
                        let label = ui.label(super::text(language, "startup.message_body_json"));
                        ui.add(
                            TextEdit::multiline(
                                self.settings
                                    .template
                                    .get_or_insert_with(|| r#"{"text":"{{text}}"}"#.into()),
                            )
                            .code_editor()
                            .desired_rows(3)
                            .desired_width(ui.available_width()),
                        )
                        .labelled_by(label.id)
                        .accessible_name(super::text(language, "startup.message_body_json"));
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
                        super::text(language, "startup.include_the_goal_text_in_messages"),
                    );
                    ui.checkbox(
                        &mut self.settings.decisions,
                        super::text(language, "startup.let_listed_approvers_reply_allow_or_deny"),
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
                                egui::Button::new(super::text(
                                    language,
                                    "startup.send_test_message",
                                )),
                            )
                            .clicked()
                            && let Ok(packet) =
                                super::test_request(&self.settings, super::optional(&self.url))
                        {
                            event = Some(Event::Test(packet));
                        }
                        if ui
                            .add_enabled(
                                can_edit && snapshot["revision"].is_string(),
                                egui::Button::new(super::text(language, "startup.save")),
                            )
                            .clicked()
                            && let Ok(packet) =
                                super::save_request(snapshot, &self.settings, self.secrets())
                        {
                            event = Some(Event::Save(packet));
                        }
                    });
                });
                if ui
                    .button(super::text(language, "startup.close"))
                    .accessible_name(super::text(language, "startup.close"))
                    .clicked()
                {
                    close_requested = true;
                }
            });
        self.open = open && !close_requested;
        event
    }
}

fn field(ui: &mut egui::Ui, language: &str, id: &'static str, value: &mut String, secret: bool) {
    let label = ui.label(super::text(language, id));
    ui.add(
        TextEdit::singleline(value)
            .password(secret)
            .desired_width(ui.available_width()),
    )
    .labelled_by(label.id)
    .accessible_name(super::text(language, id));
}

fn saved(ui: &mut egui::Ui, language: &str, present: bool, id: &'static str) {
    if present {
        ui.label(super::text(language, id));
    }
}

fn event_label(state: &str, language: &str) -> &'static str {
    super::text(
        language,
        match state {
            "plan_ready" => "startup.plan_ready",
            "waiting_approval" => "startup.candidate_ready_for_review",
            "waiting_tool_approval" => "startup.tool_request_needs_a_decision",
            "waiting_availability" => "startup.waiting_for_account_availability",
            "completed" => "startup.run_completed",
            "failed" => "startup.run_failed",
            "stopped" => "startup.stop",
            "blocked" => "room.blocked",
            _ => "startup.notify_when",
        },
    )
}
