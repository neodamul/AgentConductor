use super::super::super::{CandidateDecision, Event, State};
use eframe::egui::{self, Ui};
use serde_json::Value;

pub(super) fn show(
    state: &mut State,
    snapshot: &Value,
    current_evidence: bool,
    pending: bool,
    language: &str,
    ui: &mut Ui,
) -> Option<Event> {
    let text = |id| crate::ui::messages::in_language(language, id);
    let run_state = snapshot["report"]["state"].as_str().unwrap_or_default();
    let merge_running = snapshot["job"]["kind"] == "merge" && snapshot["job"]["state"] == "running";
    let has_candidate = snapshot["report"]["candidate"]["commit"].is_string();
    let can_approve = current_evidence && run_state == "waiting_approval";
    let can_merge = current_evidence && run_state == "approved" && !merge_running;
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
            let width = ui
                .available_width()
                .min((ui.ctx().content_rect().right() - ui.min_rect().left()).max(0.0));
            let response = ui
                .add_enabled_ui(!pending && !merge_running && enabled, |ui| {
                    ui.add_sized(egui::vec2(width, 0.0), egui::Button::new(text(key)).wrap())
                })
                .inner;
            if response.accessible_name(text(key)).clicked() {
                state.confirming_decision = Some(decision);
            }
        }
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
