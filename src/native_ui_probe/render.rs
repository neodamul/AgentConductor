use eframe::egui::{self, Color32, RichText};
use serde_json::Value;
use std::borrow::Cow;

pub fn status_color(status: &str, time: f64) -> Color32 {
    match status {
        "waiting_approval"
        | "waiting_availability"
        | "waiting_tool_approval"
        | "pending"
        | "ready" => Color32::from_rgb(218, 148, 34),
        "failed" | "blocked" | "rejected" => Color32::from_rgb(205, 55, 55),
        "running" => {
            let phase = time.rem_euclid(1.6) / 1.6;
            let pulse = 0.28 + 0.72 * (1.0 - (std::f64::consts::TAU * phase).cos()) / 2.0;
            Color32::from_rgb(
                (45.0 * pulse) as u8,
                (165.0 * pulse) as u8,
                (95.0 * pulse) as u8,
            )
        }
        "completed" | "approved" | "merged" => Color32::from_rgb(45, 165, 95),
        _ => Color32::from_rgb(125, 130, 140),
    }
}

pub fn status_label<'a>(status: &'a str, language: &str) -> Cow<'a, str> {
    let key = match status {
        "prepared" => "room.prepared",
        "ready" => "room.ready",
        "pending" => "room.pending",
        "running" => "room.running",
        "waiting_approval" => "room.waiting_approval",
        "waiting_availability" => "room.waiting_availability",
        "waiting_tool_approval" => "room.waiting_tool_approval",
        "blocked" => "room.blocked",
        "failed" => "room.failed_031a8f",
        "completed" | "merged" => "room.completed",
        "approved" => "room.approved",
        "rejected" => "room.rejected",
        "stopped" => "room.stopped",
        _ => return Cow::Borrowed(status),
    };
    Cow::Borrowed(crate::ui::messages::in_language(language, key))
}

pub fn status_colors(status: &str, dark_mode: bool) -> (Color32, Color32) {
    let (light, dark) = match status {
        "failed" | "blocked" | "rejected" => (
            (
                Color32::from_rgb(196, 49, 42),
                Color32::from_rgb(253, 236, 234),
            ),
            (
                Color32::from_rgb(224, 122, 106),
                Color32::from_rgb(58, 34, 32),
            ),
        ),
        "pending"
        | "prepared"
        | "ready"
        | "waiting_approval"
        | "waiting_availability"
        | "waiting_tool_approval" => (
            (
                Color32::from_rgb(154, 90, 0),
                Color32::from_rgb(255, 241, 220),
            ),
            (
                Color32::from_rgb(226, 192, 106),
                Color32::from_rgb(58, 47, 28),
            ),
        ),
        "running" | "completed" | "approved" | "merged" => (
            (
                Color32::from_rgb(31, 122, 58),
                Color32::from_rgb(227, 243, 232),
            ),
            (
                Color32::from_rgb(140, 201, 168),
                Color32::from_rgb(31, 51, 40),
            ),
        ),
        _ => (
            (
                Color32::from_rgb(84, 91, 100),
                Color32::from_rgb(241, 243, 245),
            ),
            (
                Color32::from_rgb(183, 190, 200),
                Color32::from_rgb(41, 44, 50),
            ),
        ),
    };
    if dark_mode { dark } else { light }
}

pub fn snapshot(snapshot: &Value, language: &str, ui: &mut egui::Ui) {
    if let Some(root) = snapshot["root"].as_str() {
        ui.label(root);
    }
    let report = snapshot["report"].as_object().map(|_| &snapshot["report"]);
    let proposal_plan = &snapshot["proposal"]["plan"];
    let plan = report
        .map(|report| &report["plan"])
        .filter(|plan| plan["tasks"].is_array())
        .or_else(|| snapshot.get("plan").filter(|plan| plan["tasks"].is_array()))
        .unwrap_or(proposal_plan);
    let goal = snapshot["config"]["goal"]
        .as_str()
        .filter(|goal| !goal.is_empty())
        .or_else(|| plan["goal"].as_str());
    if let Some(goal) = goal {
        ui.label(
            RichText::new(goal)
                .strong()
                .color(ui.visuals().text_color()),
        );
    }
    if let Some(state) = report
        .and_then(|report| report["state"].as_str())
        .or_else(|| snapshot["state"].as_str())
        .or_else(|| snapshot["run_state"].as_str())
    {
        let label = status_label(state, language);
        status_tag(
            ui,
            state,
            &format!(
                "{} · {label}",
                crate::ui::messages::in_language(language, "room.run_label")
            ),
            &crate::ui::messages::interpolate(
                crate::ui::messages::in_language(language, "room.run_status_accessible"),
                &[("value1", label.as_ref())],
            ),
        );
    }
    if let Some(tasks) = plan["tasks"].as_array() {
        let task_reports = report
            .map(|report| &report["tasks"])
            .unwrap_or(&snapshot["tasks"]);
        let run_state = report
            .and_then(|report| report["state"].as_str())
            .or_else(|| snapshot["state"].as_str())
            .or_else(|| snapshot["run_state"].as_str());
        let running = run_state == Some("running")
            || task_reports
                .as_array()
                .is_some_and(|reports| reports.iter().any(|report| report["status"] == "running"));
        if running {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(500));
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            for task in tasks {
                let id = task["id"].as_str().unwrap_or("unknown");
                let status = task_reports
                    .as_array()
                    .and_then(|reports| reports.iter().find(|report| report["id"] == id))
                    .and_then(|report| report["status"].as_str())
                    .unwrap_or("pending");
                let agent = task["agent"].as_str().unwrap_or("unassigned");
                ui.horizontal(|ui| {
                    let label = status_label(status, language);
                    let accessible = crate::ui::messages::interpolate(
                        crate::ui::messages::in_language(language, "room.task_status_accessible"),
                        &[("value1", id), ("value2", label.as_ref())],
                    );
                    status_tag(ui, status, &label, &accessible);
                    ui.label(id);
                    ui.label(format!("· {agent}"));
                });
            }
        });
    }
}

fn status_tag(ui: &mut egui::Ui, status: &str, label: &str, accessible: &str) {
    let (foreground, background) = status_colors(status, ui.visuals().dark_mode);
    let time = ui.input(|input| input.time);
    if status == "running" {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(500));
    }
    egui::Frame::new()
        .fill(background)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(4.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("●")
                        .size(9.0)
                        .color(status_color(status, time)),
                );
                ui.label(RichText::new(label).small().strong().color(foreground))
                    .accessible_name(accessible);
            });
        });
}

pub fn evidence(evidence: &Value, diff: &mut String, ui: &mut egui::Ui) {
    evidence_in_language(evidence, diff, "en", ui);
}

pub fn evidence_in_language(
    evidence: &Value,
    diff: &mut String,
    language: &str,
    ui: &mut egui::Ui,
) {
    let message = |id| crate::ui::messages::in_language(language, id);
    ui.separator();
    ui.heading(message("room.review_evidence"));
    if let Some(revision) = evidence["revision"].as_str() {
        ui.label(crate::ui::messages::interpolate(
            message("room.evidence_revision"),
            &[("value1", revision)],
        ));
    }
    if let Some(commit) = evidence["report"]["candidate"]["commit"].as_str() {
        ui.label(crate::ui::messages::interpolate(
            message("room.evidence_candidate"),
            &[("value1", commit)],
        ));
    }
    if let Some(tests) = evidence["report"]["tests"].as_array() {
        for test in tests {
            let task = test["task"].as_str().unwrap_or(message("room.unknown"));
            let exit = test["exit_code"].as_i64().map_or_else(
                || message("room.unknown").to_owned(),
                |code| code.to_string(),
            );
            let timed_out = if test["timed_out"] == true {
                message("room.timed_out")
            } else {
                ""
            };
            ui.label(crate::ui::messages::interpolate(
                message("room.evidence_test_exit"),
                &[("value1", task), ("value2", &exit), ("value3", timed_out)],
            ));
        }
    }
    if let Some(outputs) = evidence["test_output"].as_array() {
        for output in outputs {
            for (id, key) in [("room.stdout", "stdout"), ("room.stderr", "stderr")] {
                if let Some(text) = output[key].as_str().filter(|text| !text.is_empty()) {
                    let task = output["task"].as_str().unwrap_or(message("room.unknown"));
                    ui.label(crate::ui::messages::interpolate(
                        message("room.test_stream"),
                        &[("value1", task), ("value2", message(id))],
                    ));
                    ui.monospace(text);
                }
            }
        }
    }
    if evidence["truncated"] == true {
        ui.label(message("room.diff_truncated"));
    }
    egui::ScrollArea::vertical()
        .id_salt("review-diff")
        .max_height(360.0)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(diff)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(ui.available_width())
                    .interactive(false),
            );
        });
}
