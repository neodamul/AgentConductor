use super::super::{CandidateDecision, Event, State, task_activity, task_controls};
use eframe::egui::{self, Ui};
use serde_json::Value;

pub(super) fn show(
    state: &mut State,
    snapshot: &Value,
    pending: bool,
    request_details: bool,
    language: &str,
    ui: &mut Ui,
) -> Option<Event> {
    let mut event = None;
    ui.columns(3, |columns| {
        egui::ScrollArea::vertical()
            .id_salt("room-inspector")
            .show(&mut columns[2], |ui| {
                if let Some(action) =
                    task_controls::show(&mut state.task_controls, snapshot, pending, language, ui)
                {
                    event = Some(Event::TaskAction(action));
                }
                let current_evidence = state
                    .evidence
                    .as_ref()
                    .is_some_and(|evidence| evidence["revision"] == snapshot["revision"]);
                if let Some(decision) =
                    decision_controls(state, snapshot, current_evidence, pending, language, ui)
                {
                    event = Some(decision);
                }
            });
        egui::ScrollArea::vertical()
            .id_salt("room-summary")
            .show(&mut columns[0], |ui| {
                super::super::super::render::snapshot(snapshot, language, ui);
            });
        egui::ScrollArea::vertical()
            .id_salt("room-activity")
            .show(&mut columns[1], |ui| {
                if let Some(index) = state.task_controls.selected_index(snapshot)
                    && let Some(task) = snapshot["report"]["plan"]["tasks"].get(index)
                    && let Some(task_id) = task["id"].as_str()
                {
                    if request_details
                        && !pending
                        && state.details_task.as_deref() != Some(task_id)
                        && event.is_none()
                    {
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
                if state.evidence.is_none() && snapshot["report"]["candidate"].is_object() {
                    ui.add_enabled_ui(!pending, |ui| {
                        if ui
                            .button(crate::ui::messages::in_language(
                                language,
                                "room.load_diff_and_tests",
                            ))
                            .accessible_name(crate::ui::messages::in_language(
                                language,
                                "room.load_review_evidence",
                            ))
                            .clicked()
                        {
                            event = Some(Event::Evidence);
                        }
                    });
                }
                if let Some(evidence) = &state.evidence {
                    super::super::super::render::evidence_in_language(
                        evidence,
                        &mut state.diff,
                        language,
                        ui,
                    );
                }
            });
    });
    event
}

fn decision_controls(
    state: &mut State,
    snapshot: &Value,
    current_evidence: bool,
    pending: bool,
    language: &str,
    ui: &mut Ui,
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
    if matches!(state.confirming_decision, Some(CandidateDecision::Approve) if !can_approve)
        || matches!(state.confirming_decision, Some(CandidateDecision::Merge) if !can_merge)
    {
        state.confirming_decision = None;
    }
    if state.confirming_decision.is_none() {
        ui.horizontal(|ui| {
            for (decision, key, enabled) in [
                (
                    CandidateDecision::Approve,
                    "room.approve_verified_candidate",
                    can_approve,
                ),
                (CandidateDecision::Reject, "room.reject_candidate", true),
                (
                    CandidateDecision::Merge,
                    "room.merge_approved_candidate",
                    can_merge,
                ),
            ] {
                if ui
                    .add_enabled(!pending && enabled, egui::Button::new(text(key)))
                    .accessible_name(text(key))
                    .clicked()
                {
                    state.confirming_decision = Some(decision);
                }
            }
        });
    }
    let decision = state.confirming_decision?;
    let label = text(match decision {
        CandidateDecision::Approve => "room.confirm_approval_of_this_verified_diff_and_tests",
        CandidateDecision::Reject => "room.reject_this_candidate_and_send_feedback",
        CandidateDecision::Merge => "room.confirm_integrating_the_exact_approved_candidate",
    });
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
