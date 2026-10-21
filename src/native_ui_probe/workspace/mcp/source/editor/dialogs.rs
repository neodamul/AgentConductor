use super::State;
use crate::native_ui_probe::workspace::mcp::append_reference;
use eframe::egui;
use serde_json::Value;

pub(super) fn show_reference(
    state: &mut State,
    goal: &mut String,
    language: &str,
    can_edit: bool,
    ui: &mut egui::Ui,
) {
    let Some(kind) = state.reference_kind else {
        return;
    };
    let text = |id| crate::ui::messages::in_language(language, id);
    let mut open = true;
    let mut close_requested = false;
    egui::Window::new(text("mcp.add_a_document_reference"))
        .open(&mut open)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(520.0)
        .show(ui.ctx(), |ui| {
            ui.label(text("mcp.enter_a_jira_issue_key_or_numeric_confluence"));
            let input = ui.add(
                egui::TextEdit::singleline(&mut state.reference_id)
                    .hint_text(text("mcp.emm_52599_or_53553565")),
            );
            input.accessible_name(text("mcp.add_reference"));
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        can_edit && !state.reference_id.trim().is_empty(),
                        egui::Button::new(text("mcp.add_reference")),
                    )
                    .accessible_name(text("mcp.add_reference"))
                    .clicked()
                {
                    match append_reference(goal, kind, state.reference_id.trim()) {
                        Ok(()) => {
                            state.reference_kind = None;
                            state.error = None;
                        }
                        Err(error) => state.error = Some(error),
                    }
                }
                if ui
                    .button(text("startup.close"))
                    .accessible_name(text("startup.close"))
                    .clicked()
                {
                    close_requested = true;
                }
            });
        });
    if !open || close_requested {
        state.reference_kind = None;
    }
}

pub(super) fn show_status(snapshot: &Value, language: &str, ui: &mut egui::Ui) {
    if let Some(applied) = snapshot.get("imports").filter(|value| value.is_object()) {
        let count = applied["selected"]
            .as_array()
            .map_or(0, Vec::len)
            .to_string();
        ui.label(crate::ui::messages::render(
            "mcp.applied_sources",
            &[("value1", &count)],
        ));
    }
    if crate::mcp::references(snapshot["config"]["goal"].as_str().unwrap_or_default())
        .is_ok_and(|references| !references.is_empty())
        && snapshot["config"]["coordinator"]
            .as_str()
            .is_none_or(str::is_empty)
    {
        ui.label(crate::ui::messages::in_language(
            language,
            "mcp.coordinator_required",
        ));
    }
}
