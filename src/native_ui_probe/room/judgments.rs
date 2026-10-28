use eframe::egui;
use serde_json::Value;

#[cfg(test)]
#[path = "../../../tests/native_ui_probe/room_judgments.rs"]
mod tests;

pub fn label(row: &Value, language: &str) -> String {
    let kind = row["kind"].as_str().unwrap_or_default();
    let verdict = row["judgment"].as_str().unwrap_or_default();
    let id = match (kind, verdict) {
        ("tool", "allow") => "room.judgment_tool_allow",
        ("tool", "deny") => "room.judgment_tool_deny",
        ("tool", _) => "room.judgment_tool_undecided",
        ("review", "accept") => "room.judgment_review_accept",
        ("review", "revise") => "room.judgment_review_revise",
        ("review", "blocked") => "room.judgment_review_blocked",
        ("amendment", "applied") => "room.judgment_amendment_applied",
        ("amendment", "undone") => "room.judgment_amendment_undone",
        ("amendment", "escalated") => "room.judgment_amendment_escalated",
        ("test_command", _) => "room.judgment_test_command_replaced",
        ("timeout", _) => "room.judgment_timeout_resume",
        ("recovery", _) => "room.judgment_recovery",
        ("handoff", _) => "room.judgment_handoff_reassigned",
        ("lesson", _) => "room.judgment_lesson_recorded",
        ("gate", "passed") => "room.judgment_gate_passed",
        ("goal_drift", _) => "room.judgment_goal_drift",
        ("request", "queued") => "room.judgment_request_queued",
        ("request", "processing") => "room.judgment_request_processing",
        ("request", "completed") => "room.judgment_request_completed",
        ("request", "failed") => "room.judgment_request_failed",
        ("request", _) => "room.judgment_coordinator_request",
        _ => "room.judgment_other",
    };
    crate::ui::messages::interpolate(
        crate::ui::messages::in_language(language, id),
        &[
            ("value1", if kind == "recovery" { verdict } else { kind }),
            ("value2", verdict),
        ],
    )
}

pub fn entries(rows: &Value, language: &str, ui: &mut egui::Ui) -> Option<String> {
    let mut undo = None;
    for row in rows.as_array().into_iter().flatten() {
        ui.group(|ui| {
            let title = label(row, language);
            ui.label(egui::RichText::new(&title).strong())
                .accessible_name(title);
            if let Some(task) = row["task"].as_str().filter(|text| !text.is_empty()) {
                ui.label(task).accessible_name(task);
            }
            for field in ["detail", "summary"] {
                if let Some(text) = row[field].as_str().filter(|text| !text.is_empty()) {
                    ui.label(text).accessible_name(text);
                }
            }
            if let Some(applied) = row["applied"].as_bool() {
                let text = crate::ui::messages::in_language(
                    language,
                    if applied {
                        "room.judgment_applied"
                    } else {
                        "room.judgment_not_applied"
                    },
                );
                ui.label(text).accessible_name(text);
            }
            if row["kind"] == "amendment" && row["judgment"] == "applied" && row["undone"] != true {
                let label = crate::ui::messages::in_language(language, "room.undo_amendment");
                if ui.button(label).clicked() {
                    undo = row["id"].as_str().map(str::to_owned);
                }
            }
        });
    }
    undo
}

pub fn view(data: &Value, language: &str, ui: &mut egui::Ui) -> Option<String> {
    let loaded = data["judgments"].as_array().map_or(0, Vec::len);
    if data["decision_count"]
        .as_u64()
        .is_some_and(|count| count > loaded as u64)
    {
        ui.label(crate::ui::messages::interpolate(
            crate::ui::messages::in_language(language, "room.judgments_limited"),
            &[
                ("value1", &loaded.to_string()),
                ("value2", &data["decision_count"].to_string()),
            ],
        ));
    }
    let undo = entries(&data["judgments"], language, ui);
    for item in data["history"].as_array().into_iter().flatten() {
        if let Some(summary) = item["summary"].as_str().filter(|s| !s.is_empty()) {
            ui.label(summary).accessible_name(summary);
        }
    }
    undo
}

impl crate::native_ui_probe::Probe {
    pub(in crate::native_ui_probe) fn undo_amendment(&mut self, id: String) {
        let result = self.snapshot.as_ref().ok_or_else(|| "Agent Room snapshot is unavailable".to_owned()).and_then(|snapshot| {
            let revision = snapshot["revision"].as_str().ok_or("current run revision is unavailable")?;
            serde_json::to_vec(&serde_json::json!({"kind":"action","revision":revision,"action":{"kind":"amendment_undo","id":id}})).map_err(|error| error.to_string())
        }).and_then(|packet| self.worker.as_ref().ok_or_else(|| "Agent Room controller is unavailable".to_owned())?.send(&packet));
        match result {
            Ok(()) => {
                self.pending = Some(crate::native_ui_probe::Pending::Action);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }
}

impl crate::native_ui_probe::Probe {
    pub(in crate::native_ui_probe) fn load_coordinator_judgments(&mut self) {
        if let Some(view) = self.room_view.as_mut() {
            view.coordinator_selected = true;
        }
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Agent Room controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"coordinator"}"#));
        match result {
            Ok(()) => self.pending = Some(crate::native_ui_probe::Pending::Coordinator),
            Err(error) => self.error = Some(error),
        }
    }

    pub(in crate::native_ui_probe) fn coordinator_received(&mut self, snapshot: Value) {
        if let Some(view) = self.room_view.as_mut() {
            view.coordinator_judgments = Some(snapshot);
        }
        self.pending = None;
        self.error = None;
    }

    pub(in crate::native_ui_probe) fn refresh_room_details(&mut self) {
        if self.snapshot.as_ref().is_some_and(|snapshot| {
            snapshot["tool_requests"]
                .as_array()
                .is_some_and(|requests| {
                    requests.iter().any(|request| {
                        matches!(request["status"].as_str(), Some("pending" | "approved"))
                    })
                })
        }) {
            return;
        }
        let changed = self
            .room_view
            .as_ref()
            .filter(|view| view.coordinator_selected)
            .and_then(|view| view.coordinator_judgments.as_ref())
            .zip(self.snapshot.as_ref())
            .is_some_and(|(data, snapshot)| data["decision_count"] != snapshot["decision_count"]);
        if changed {
            self.load_coordinator_judgments();
        } else if !self
            .room_view
            .as_ref()
            .is_some_and(|view| view.coordinator_selected)
        {
            self.refresh_selected_task_details();
        }
    }
}
