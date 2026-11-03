use super::task_controls::Action;
use eframe::egui;
use serde_json::Value;

pub fn show(
    draft: &mut String,
    judgments: Option<&Value>,
    pending: bool,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Action> {
    let text = |id| crate::ui::messages::in_language(language, id);
    ui.separator();
    ui.heading(text("room.coordinator_feedback"));
    let recipient = judgments
        .and_then(|data| data["coordinator"].as_str())
        .filter(|id| !id.is_empty());
    let recipient = recipient.unwrap_or(text("room.coordinator_unavailable"));
    ui.label(crate::ui::messages::interpolate(
        text("room.coordinator_recipient"),
        &[("value1", recipient)],
    ));
    if let Some(row) = judgments
        .and_then(|data| data["judgments"].as_array())
        .into_iter()
        .flatten()
        .rev()
        .find(|row| row["kind"] == "request")
    {
        let status = super::judgments::label(row, language);
        ui.label(&status).accessible_name(status);
    }
    let label = ui.label(text("room.coordinator_request_label"));
    ui.add_enabled(
        !pending,
        egui::TextEdit::multiline(draft)
            .desired_rows(3)
            .desired_width(ui.available_width()),
    )
    .labelled_by(label.id)
    .accessible_name(text("room.coordinator_request_label"));
    let enabled = !pending
        && recipient != text("room.coordinator_unavailable")
        && !draft.trim().is_empty()
        && draft.len() <= 8 * 1024
        && !draft.contains('\0');
    if ui
        .add_enabled(enabled, egui::Button::new(text("room.send_to_coordinator")))
        .accessible_name(text("room.send_to_coordinator"))
        .clicked()
    {
        Some(Action::CoordinatorRequest {
            message: draft.clone(),
        })
    } else {
        None
    }
}
