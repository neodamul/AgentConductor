use super::{Event, State};
use eframe::egui::{self, Color32};
use serde_json::Value;

mod columns;
mod human_review;
mod stages;
mod verification;

pub fn show(
    state: &mut State,
    snapshot: Option<&Value>,
    pending: bool,
    error: Option<&str>,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let text = |id| crate::ui::messages::in_language(language, id);
    let mut event = None;
    ui.horizontal(|ui| {
        ui.heading(text("room.agent_room"));
        ui.label(
            egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                .small()
                .weak(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(!pending, egui::Button::new(text("room.team_briefing")))
                .accessible_name(text("room.load_read_only_team_briefing"))
                .clicked()
            {
                event = Some(Event::Briefing);
            }
            if ui
                .add_enabled(
                    !pending,
                    egui::Button::new(format!("↻ {}", text("room.refresh"))),
                )
                .accessible_name(text("room.refresh_run"))
                .clicked()
            {
                event = Some(Event::Refresh);
            }
        });
    });
    if let Some(goal) = snapshot
        .and_then(|value| value["report"]["plan"]["goal"].as_str())
        .filter(|goal| !goal.trim().is_empty())
    {
        ui.label(
            egui::RichText::new(text("room.current_goal"))
                .small()
                .strong()
                .weak(),
        );
        ui.heading(goal);
    }
    if let Some(snapshot) = snapshot {
        event = super::status_banner::show(state, snapshot, pending, language, ui);
        stages::show(snapshot, language, ui);
    }
    if let Some(briefing) = &state.briefing {
        super::super::room_briefing::show(briefing, language, ui);
    }
    let run_state = snapshot
        .and_then(|value| value["report"]["state"].as_str())
        .unwrap_or_default();
    let stoppable = matches!(
        run_state,
        "running" | "ready" | "waiting_availability" | "waiting_tool_approval" | "stopped"
    );
    if !stoppable {
        state.confirming_stop = false;
    }
    if !state.confirming_stop
        && stoppable
        && ui
            .add_enabled(!pending, egui::Button::new(text("room.stop_run")))
            .accessible_name(text("room.stop_run"))
            .clicked()
    {
        state.confirming_stop = true;
        if state.stop_reason.is_empty() {
            state.stop_reason = text("room.operator_requested_stop").into();
        }
    }
    if state.confirming_stop && stoppable {
        ui.separator();
        ui.label(text("room.stop_all_active_tasks_and_tests_retain_their"));
        let label = ui.label(text("room.stop_reason"));
        ui.add_enabled(
            !pending,
            egui::TextEdit::singleline(&mut state.stop_reason).desired_width(ui.available_width()),
        )
        .labelled_by(label.id)
        .accessible_name(text("room.stop_reason"));
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!pending, egui::Button::new(text("room.cancel_stop")))
                .accessible_name(text("room.cancel_stop"))
                .clicked()
            {
                state.confirming_stop = false;
            }
            if ui
                .add_enabled(
                    !pending
                        && !state.stop_reason.trim().is_empty()
                        && state.stop_reason.len() <= 16 * 1024,
                    egui::Button::new(text("room.confirm_stop")),
                )
                .accessible_name(text("room.confirm_stop"))
                .clicked()
            {
                state.confirming_stop = false;
                event = Some(Event::Stop(state.stop_reason.clone()));
            }
        });
    }
    if let Some(action) =
        super::super::room_actions::controls(state, snapshot, pending, language, ui)
    {
        event = Some(action);
    }
    if let Some(action) =
        super::super::room_actions::tool_controls(state, snapshot, pending, language, ui)
    {
        event = Some(action);
    }
    if let Some(snapshot) = snapshot {
        let request_details = event.is_none();
        event = event.or(columns::show(
            state,
            snapshot,
            pending,
            request_details,
            language,
            ui,
        ));
    } else {
        ui.label(text("room.connecting_to_local_run"));
    }
    if pending {
        ui.spinner();
    }
    if let Some(notice) = state.notice.as_deref() {
        ui.label(notice);
    }
    if let Some(error) = error {
        ui.colored_label(Color32::DARK_RED, error);
    }
    event
}
