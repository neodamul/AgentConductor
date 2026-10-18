use super::render::clip;
use super::*;

pub(super) fn logs(details: &Value, language: &str, ui: &mut egui::Ui) {
    for (key, title, empty) in [
        (
            "stdout",
            "room.stdout_recent_local_tail",
            "room.no_output_yet",
        ),
        (
            "stderr",
            "room.stderr_recent_local_tail",
            "room.no_errors_logged",
        ),
    ] {
        ui.label(RichText::new(crate::ui::messages::in_language(language, title)).strong())
            .accessible_name(crate::ui::messages::in_language(language, title));
        let value = details[key]
            .as_str()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| crate::ui::messages::in_language(language, empty));
        let value = clip(value, 16 * 1024);
        ui.label(RichText::new(&value).monospace())
            .accessible_name(value);
    }
}

pub(super) fn lessons(task_id: &str, details: &Value, language: &str, ui: &mut egui::Ui) {
    let text = |key| crate::ui::messages::in_language(language, key);
    let items = details["knowledge"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|item| item["task"] == task_id)
        .collect::<Vec<_>>();
    if items.is_empty() {
        ui.label(text(
            "room.no_reviewed_lessons_for_this_task_draft_observations",
        ))
        .accessible_name(text(
            "room.no_reviewed_lessons_for_this_task_draft_observations",
        ));
        return;
    }
    for item in items {
        let lesson = &item["lesson"];
        ui.group(|ui| {
            let context = lesson["context"].as_str().unwrap_or_default();
            ui.heading(context).accessible_name(context);
            let confidence = format!(
                "{} · {}",
                item["status"].as_str().unwrap_or_default(),
                lesson["confidence"].as_f64().unwrap_or_default()
            );
            ui.label(&confidence).accessible_name(confidence);
            let response = lesson["response"].as_str().unwrap_or_default();
            ui.label(response).accessible_name(response);
            for source in lesson["sources"].as_array().into_iter().flatten() {
                let content = format!(
                    "{} · {}",
                    source["path"].as_str().unwrap_or_default(),
                    source["sha256"].as_str().unwrap_or_default()
                );
                ui.label(&content).accessible_name(content);
            }
            let revalidation = format!(
                "{}{}",
                text("room.revalidate"),
                lesson["revalidate"].as_str().unwrap_or_default()
            );
            ui.label(&revalidation).accessible_name(revalidation);
        });
    }
}
