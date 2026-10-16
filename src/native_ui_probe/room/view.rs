use super::{CandidateDecision, Event, State, task_activity, task_controls};
use eframe::egui::{self, Color32};
use serde_json::Value;

pub fn show(
    state: &mut State,
    snapshot: Option<&Value>,
    pending: bool,
    error: Option<&str>,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let text = |id| crate::ui::messages::in_language(language, id);
    ui.horizontal(|ui| {
        ui.heading(text("room.agent_room"));
        ui.label(
            egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                .small()
                .weak(),
        );
    });
    let mut event = ui
        .add_enabled(
            !pending,
            egui::Button::new(format!("↻ {}", text("room.refresh"))),
        )
        .accessible_name(text("room.refresh_run"))
        .clicked()
        .then_some(Event::Refresh);
    if ui
        .add_enabled(!pending, egui::Button::new(text("room.team_briefing")))
        .accessible_name(text("room.load_read_only_team_briefing"))
        .clicked()
    {
        event = Some(Event::Briefing);
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
        super::super::render::snapshot(snapshot, language, ui);
        if let Some(action) =
            task_controls::show(&mut state.task_controls, snapshot, pending, language, ui)
        {
            event = Some(Event::TaskAction(action));
        }
        if let Some(index) = state.task_controls.selected_index(snapshot)
            && let Some(task) = snapshot["report"]["plan"]["tasks"].get(index)
            && let Some(task_id) = task["id"].as_str()
        {
            if !pending && state.details_task.as_deref() != Some(task_id) && event.is_none() {
                event = Some(Event::TaskDetails(index));
            }
            let provider = snapshot["report"]["plan"]["agents"]
                .as_array()
                .and_then(|agents| agents.iter().find(|agent| agent["id"] == task["agent"]))
                .and_then(|agent| agent["parser"].as_str())
                .unwrap_or("custom");
            task_activity::show(
                &mut state.activity_tab,
                task_id,
                provider,
                task,
                state.task_details.as_ref(),
                language,
                ui,
            );
        }
    } else {
        ui.label(text("room.connecting_to_local_run"));
    }
    if state.evidence.is_none()
        && snapshot.is_some_and(|snapshot| snapshot["report"]["candidate"].is_object())
    {
        ui.add_enabled_ui(!pending, |ui| {
            if ui
                .button(text("room.load_diff_and_tests"))
                .accessible_name(text("room.load_review_evidence"))
                .clicked()
            {
                event = Some(Event::Evidence);
            }
        });
    }
    if let Some(evidence) = &state.evidence {
        super::super::render::evidence_in_language(evidence, &mut state.diff, language, ui);
    }
    let current_evidence = state.evidence.as_ref().is_some_and(|evidence| {
        snapshot.is_some_and(|snapshot| evidence["revision"] == snapshot["revision"])
    });
    if let Some(snapshot) = snapshot
        && let Some(decision) =
            decision_controls(state, snapshot, current_evidence, pending, language, ui)
    {
        event = Some(decision);
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

fn decision_controls(
    state: &mut State,
    snapshot: &Value,
    current_evidence: bool,
    pending: bool,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let text = |id| crate::ui::messages::in_language(language, id);
    let run_state = snapshot["report"]["state"].as_str().unwrap_or_default();
    let has_candidate = snapshot["report"]["candidate"]["commit"].is_string();
    let can_approve = current_evidence && run_state == "waiting_approval";
    let can_merge = current_evidence && run_state == "approved";
    if !has_candidate || matches!(run_state, "merged" | "completed" | "rejected") {
        state.confirming_decision = None;
        return None;
    }
    if matches!(
        state.confirming_decision,
        Some(CandidateDecision::Approve) if !can_approve
    ) || matches!(
        state.confirming_decision,
        Some(CandidateDecision::Merge) if !can_merge
    ) {
        state.confirming_decision = None;
    }
    if state.confirming_decision.is_none() {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !pending && can_approve,
                    egui::Button::new(text("room.approve_verified_candidate")),
                )
                .accessible_name(text("room.approve_verified_candidate"))
                .clicked()
            {
                state.confirming_decision = Some(CandidateDecision::Approve);
            }
            if ui
                .add_enabled(!pending, egui::Button::new(text("room.reject_candidate")))
                .accessible_name(text("room.reject_candidate"))
                .clicked()
            {
                state.confirming_decision = Some(CandidateDecision::Reject);
            }
            if ui
                .add_enabled(
                    !pending && can_merge,
                    egui::Button::new(text("room.merge_approved_candidate")),
                )
                .accessible_name(text("room.merge_approved_candidate"))
                .clicked()
            {
                state.confirming_decision = Some(CandidateDecision::Merge);
            }
        });
    }
    let decision = state.confirming_decision?;
    let label = match decision {
        CandidateDecision::Approve => text("room.confirm_approval_of_this_verified_diff_and_tests"),
        CandidateDecision::Reject => text("room.reject_this_candidate_and_send_feedback"),
        CandidateDecision::Merge => text("room.confirm_integrating_the_exact_approved_candidate"),
    };
    ui.separator();
    ui.label(label).accessible_name(label);
    if decision == CandidateDecision::Reject {
        let label = ui.label(text("room.rejection_reason"));
        ui.add_enabled(
            !pending,
            egui::TextEdit::singleline(&mut state.reject_reason),
        )
        .labelled_by(label.id)
        .accessible_name(text("room.rejection_reason"));
    }
    let mut event = None;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                !pending,
                egui::Button::new(text("room.cancel_candidate_decision")),
            )
            .accessible_name(text("room.cancel_candidate_decision"))
            .clicked()
        {
            state.confirming_decision = None;
        }
        let valid_reason = decision != CandidateDecision::Reject
            || !state.reject_reason.trim().is_empty() && state.reject_reason.len() <= 16 * 1024;
        if ui
            .add_enabled(
                !pending && valid_reason,
                egui::Button::new(text("room.confirm_candidate_decision")),
            )
            .accessible_name(text("room.confirm_candidate_decision"))
            .clicked()
        {
            event = Some(Event::Decision(decision, state.reject_reason.clone()));
            state.confirming_decision = None;
        }
    });
    event
}
