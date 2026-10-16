use super::{Pending, Probe};
use eframe::egui;
use serde_json::Value;

pub(super) fn show(briefing: &Value, ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("Team briefing · read only")
        .default_open(true)
        .show(ui, |ui| {
            ui.label(format!(
                "Captured · {} · {}",
                briefing["state"].as_str().unwrap_or("unknown"),
                format_time(briefing["observed_at_ms"].as_i64(), 1_000)
            ));
            let sessions = briefing["sessions"].as_array();
            if let Some(sessions) = sessions {
                egui::ScrollArea::vertical()
                    .max_height(240.0)
                    .show(ui, |ui| {
                        for session in sessions {
                            ui.group(|ui| {
                                ui.strong(format!(
                                    "{} · {} · {}",
                                    field(session, "agent"),
                                    field(session, "task"),
                                    field(session, "state")
                                ));
                                ui.label(field(session, "work"));
                                ui.label(format!(
                                    "Session · {} · Model · {} · Attempt · {}",
                                    field(session, "session_id"),
                                    field(session, "model"),
                                    session["attempt"]
                                        .as_u64()
                                        .map(|attempt| attempt.to_string())
                                        .unwrap_or_else(|| "unknown".into())
                                ));
                                ui.label(format!(
                                    "Process · {}",
                                    field(&session["process"], "state")
                                ));
                                if let Some(time) = session["process"]["observed_at_ms"].as_i64() {
                                    ui.label(format!(
                                        "Process observed · {}",
                                        format_time(Some(time), 1_000)
                                    ));
                                }
                                let activity = &session["last_activity"];
                                if activity.is_object() {
                                    ui.label(format!(
                                        "Last activity · {} · {} · {}",
                                        field(activity, "kind"),
                                        field(activity, "freshness"),
                                        field(activity, "text")
                                    ));
                                    if let Some(time) = activity["observed_at_ms"].as_i64() {
                                        ui.label(format!(
                                            "Activity observed · {}",
                                            format_time(Some(time), 1_000)
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
                                                format!(
                                                    "Available · {}",
                                                    format_time(Some(time), 1)
                                                )
                                            })
                                            .unwrap_or_else(|| "Availability reset unknown".into()),
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
                    "No recorded approval requests".into()
                } else {
                    format!(
                        "Review requests · {}",
                        approvals
                            .iter()
                            .map(|approval| format!(
                                "{} · {} · {}",
                                field(approval, "kind"),
                                field(approval, "task"),
                                field(approval, "status")
                            ))
                            .collect::<Vec<_>>()
                            .join("; ")
                    )
                });
            }
        });
}

fn field(value: &Value, key: &str) -> String {
    value[key]
        .as_str()
        .filter(|text| !text.is_empty())
        .unwrap_or("unknown")
        .to_owned()
}

fn format_time(timestamp: Option<i64>, units_per_second: i64) -> String {
    timestamp
        .and_then(|value| time::OffsetDateTime::from_unix_timestamp(value / units_per_second).ok())
        .map(|value| {
            value
                .to_offset(time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC))
                .to_string()
        })
        .unwrap_or_else(|| "time unavailable".into())
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
