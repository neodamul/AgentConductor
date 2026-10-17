use super::{Pending, Probe};
use eframe::egui;
use serde_json::Value;

pub(super) fn show(briefing: &Value, language: &str, ui: &mut egui::Ui) {
    let text = |id| crate::ui::messages::in_language(language, id);
    egui::CollapsingHeader::new(text("room.team_briefing_read_only"))
        .default_open(true)
        .show(ui, |ui| {
            let state = super::render::status_label(
                briefing["state"].as_str().unwrap_or(text("room.unknown")),
                language,
            );
            let observed = format_time(briefing["observed_at_ms"].as_i64(), 1_000, language);
            ui.label(crate::ui::messages::interpolate(
                text("room.captured_read_only"),
                &[("value1", state.as_ref()), ("value2", &observed)],
            ));
            let sessions = briefing["sessions"].as_array();
            if let Some(sessions) = sessions {
                egui::ScrollArea::vertical()
                    .max_height(240.0)
                    .show(ui, |ui| {
                        for session in sessions {
                            ui.group(|ui| {
                                let agent = field(session, "agent", language);
                                let task = field(session, "task", language);
                                let status = super::render::status_label(
                                    session["state"].as_str().unwrap_or(text("room.unknown")),
                                    language,
                                );
                                ui.strong(format!("{} · {} · {}", agent, task, status));
                                ui.label(field(session, "work", language));
                                let session_id = field(session, "session_id", language);
                                let model = field(session, "model", language);
                                let attempt = session["attempt"]
                                    .as_u64()
                                    .map(|attempt| attempt.to_string())
                                    .unwrap_or_else(|| text("room.unknown").into());
                                ui.label(crate::ui::messages::interpolate(
                                    text("room.session_model_attempt"),
                                    &[
                                        ("value1", &session_id),
                                        ("value2", &model),
                                        ("value3", &attempt),
                                    ],
                                ));
                                let process_state = super::render::status_label(
                                    session["process"]["state"]
                                        .as_str()
                                        .unwrap_or(text("room.unknown")),
                                    language,
                                );
                                let process_time = format_time(
                                    session["process"]["observed_at_ms"].as_i64(),
                                    1_000,
                                    language,
                                );
                                ui.label(crate::ui::messages::interpolate(
                                    text("room.process_observed"),
                                    &[
                                        ("value1", process_state.as_ref()),
                                        ("value2", &process_time),
                                    ],
                                ));
                                let activity = &session["last_activity"];
                                if activity.is_object() {
                                    let kind = field(activity, "kind", language);
                                    let freshness = field(activity, "freshness", language);
                                    let detail = field(activity, "text", language);
                                    ui.label(crate::ui::messages::interpolate(
                                        text("room.last_activity"),
                                        &[
                                            ("value1", &kind),
                                            ("value2", &freshness),
                                            ("value3", &format!(" · {detail}")),
                                        ],
                                    ));
                                    if let Some(time) = activity["observed_at_ms"].as_i64() {
                                        let time = format_time(Some(time), 1_000, language);
                                        ui.label(crate::ui::messages::interpolate(
                                            text("room.activity_observed"),
                                            &[("value1", &time)],
                                        ));
                                    }
                                }
                                let restriction = &session["restriction"];
                                if restriction.is_object() {
                                    ui.colored_label(
                                        egui::Color32::YELLOW,
                                        restriction["available_at"]
                                            .as_i64()
                                            .map(|time| {
                                                let time = format_time(Some(time), 1, language);
                                                crate::ui::messages::interpolate(
                                                    text("room.available"),
                                                    &[("value1", &time)],
                                                )
                                            })
                                            .unwrap_or_else(|| {
                                                text("room.availability_reset_unknown").into()
                                            }),
                                    );
                                }
                                for key in ["attention", "next_action"] {
                                    if let Some(text) =
                                        session[key].as_str().filter(|v| !v.is_empty())
                                    {
                                        ui.label(text);
                                    }
                                }
                            });
                        }
                    });
            }
            let approvals = briefing["approvals"].as_array();
            if let Some(approvals) = approvals {
                ui.label(if approvals.is_empty() {
                    text("room.no_recorded_approval_requests").into()
                } else {
                    format!(
                        "{}{}",
                        text("room.review_requests"),
                        approvals
                            .iter()
                            .map(|approval| {
                                let kind = field(approval, "kind", language);
                                let task = field(approval, "task", language);
                                let status = super::render::status_label(
                                    approval["status"].as_str().unwrap_or(text("room.unknown")),
                                    language,
                                );
                                format!("{kind} · {task} · {status}")
                            })
                            .collect::<Vec<_>>()
                            .join("; ")
                    )
                });
            }
        });
}

fn field(value: &Value, key: &str, language: &str) -> String {
    value[key]
        .as_str()
        .filter(|text| !text.is_empty())
        .unwrap_or(crate::ui::messages::in_language(language, "room.unknown"))
        .to_owned()
}

fn format_time(timestamp: Option<i64>, units_per_second: i64, language: &str) -> String {
    timestamp
        .and_then(|value| time::OffsetDateTime::from_unix_timestamp(value / units_per_second).ok())
        .map(|value| {
            value
                .to_offset(time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC))
                .to_string()
        })
        .unwrap_or_else(|| {
            crate::ui::messages::in_language(language, "room.time_unavailable").into()
        })
}

impl Probe {
    pub(super) fn load_briefing(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Agent Room controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"briefing"}"#));
        match result {
            Ok(()) => {
                self.pending = Some(Pending::Briefing);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }
}
