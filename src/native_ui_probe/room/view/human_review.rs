use super::super::task_activity::Tab;
use super::{Event, State};
use eframe::egui::{self, Ui};
use serde_json::Value;

pub(super) fn show(
    state: &mut State,
    snapshot: &Value,
    pending: bool,
    language: &str,
    ui: &mut Ui,
) -> Option<Event> {
    let report = &snapshot["report"];
    let status = report["state"].as_str()?;
    let text = |key| crate::ui::messages::in_language(language, key);
    ui.separator();
    ui.heading(text("room.human_review"));
    let mut event = None;
    if report["candidate"]["commit"]
        .as_str()
        .is_some_and(|commit| !commit.is_empty())
    {
        let current = state
            .evidence
            .as_ref()
            .is_some_and(|evidence| evidence["revision"] == snapshot["revision"]);
        if ui
            .add_enabled(
                !pending,
                egui::Button::new(format!("{} ↗", text("room.review_candidate"))),
            )
            .accessible_name(text("room.review_candidate"))
            .clicked()
        {
            state.activity_tab = Tab::Review;
            if !current {
                event = Some(Event::Evidence);
            }
        }
    }
    let requests = snapshot["tool_requests"].as_array().map_or(0, Vec::len);
    ui.label(format!("{} · {requests}", text("room.tool_requests")));
    ui.label(super::super::super::render::status_label(status, language));
    event
}
