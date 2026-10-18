use eframe::egui::{self, RichText};
use serde_json::Value;

pub(super) fn show(
    controls: &mut super::task_controls::State,
    snapshot: &Value,
    language: &str,
    ui: &mut egui::Ui,
) {
    let text = |key| crate::ui::messages::in_language(language, key);
    let report = &snapshot["report"];
    let state = report["state"].as_str().unwrap_or("unknown");
    let tasks = report["tasks"].as_array();
    let total = report["plan"]["tasks"].as_array().map_or(0, Vec::len);
    let completed = tasks.map_or(0, |items| {
        items
            .iter()
            .filter(|task| task["status"] == "completed")
            .count()
    });
    let error = report["error"]
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            tasks.and_then(|items| {
                items.iter().find(|task| task["status"] == "failed")?["error"].as_str()
            })
        });
    let (foreground, background) =
        super::super::render::status_colors(state, ui.visuals().dark_mode);
    let label = super::super::render::status_label(state, language);
    egui::Frame::new()
        .fill(background)
        .stroke(egui::Stroke::new(1.0, foreground))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(text("room.run_status"))
                        .small()
                        .strong()
                        .color(foreground),
                );
                ui.label(RichText::new(format!("● {}", label)).strong().color(
                    super::super::render::status_color(state, ui.input(|input| input.time)),
                ));
                ui.label(crate::ui::messages::interpolate(
                    text("room.tasks_completed"),
                    &[
                        ("value1", &completed.to_string()),
                        ("value2", &total.to_string()),
                    ],
                ));
            });
            if let Some(error) = error {
                ui.colored_label(
                    super::super::render::status_color("failed", ui.input(|input| input.time)),
                    format!("{}{}", text("room.last_error"), error),
                );
            }
            if state == "failed"
                && controls_can_draft(snapshot)
                && ui
                    .button(text("room.fix_with_feedback"))
                    .accessible_name(text("room.fix_with_feedback"))
                    .clicked()
            {
                controls.draft_failed_task_feedback(snapshot);
            }
        });
}

fn controls_can_draft(snapshot: &Value) -> bool {
    snapshot["report"]["plan"]["tasks"]
        .as_array()
        .is_some_and(|tasks| {
            tasks.iter().any(|task| {
                task["id"].as_str().is_some_and(|id| {
                    snapshot["report"]["tests"].as_array().is_some_and(|tests| {
                        tests.iter().any(|test| {
                            test["task"] == id
                                && (test["exit_code"].as_i64().is_some_and(|code| code != 0)
                                    || test["timed_out"] == true)
                        })
                    }) || snapshot["report"]["tasks"]
                        .as_array()
                        .is_some_and(|reports| {
                            reports
                                .iter()
                                .any(|report| report["id"] == id && report["status"] == "failed")
                        })
                })
            })
        })
}
