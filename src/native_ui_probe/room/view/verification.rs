use eframe::egui::{self, RichText};
use serde_json::Value;

pub(super) fn show(snapshot: &Value, language: &str, ui: &mut egui::Ui) {
    let report = &snapshot["report"];
    let Some(tests) = report["tests"].as_array().filter(|tests| !tests.is_empty()) else {
        return;
    };
    let total = report["plan"]["tasks"].as_array().map_or(0, Vec::len);
    if total == 0 {
        return;
    }
    let passed = tests
        .iter()
        .filter(|test| test["exit_code"] == 0 && test["timed_out"] != true)
        .count();
    let failed = tests.iter().any(|test| {
        test["timed_out"] == true || test["exit_code"].as_i64().is_some_and(|code| code != 0)
    });
    let state = if failed {
        "failed"
    } else if tests.len() == total && passed == total {
        "completed"
    } else {
        "running"
    };
    let text = |key| crate::ui::messages::in_language(language, key);
    ui.separator();
    ui.heading(text("room.integration_verification"));
    let status = super::super::super::render::status_label(state, language);
    ui.label(
        RichText::new(crate::ui::messages::interpolate(
            text("room.integration_tests_progress"),
            &[
                ("value1", &passed.to_string()),
                ("value2", &total.to_string()),
            ],
        ))
        .color(super::super::super::render::status_color(
            state,
            ui.input(|input| input.time),
        )),
    );
    ui.label(status);
}
