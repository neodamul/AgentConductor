use eframe::egui;
use serde_json::Value;

use super::{Event, State};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateDecision {
    Approve,
    Reject,
    Merge,
}

pub fn candidate_decision_request(
    snapshot: &Value,
    decision: CandidateDecision,
    reason: &str,
) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .ok_or("current run revision is unavailable")?;
    let state = snapshot["report"]["state"]
        .as_str()
        .ok_or("current run state is unavailable")?;
    let candidate = snapshot["report"]["candidate"]["commit"]
        .as_str()
        .filter(|commit| !commit.is_empty())
        .ok_or("no candidate is available")?;
    let action = match decision {
        CandidateDecision::Approve if state == "waiting_approval" => {
            serde_json::json!({"kind":"approve","sha":candidate})
        }
        CandidateDecision::Approve => return Err("candidate is not waiting for approval".into()),
        CandidateDecision::Merge if state == "approved" => {
            serde_json::json!({"kind":"merge","sha":candidate})
        }
        CandidateDecision::Merge => return Err("candidate must be approved before merge".into()),
        CandidateDecision::Reject if !reason.trim().is_empty() && reason.len() <= 16 * 1024 => {
            serde_json::json!({"kind":"reject","reason":reason})
        }
        CandidateDecision::Reject => {
            return Err("rejection needs a nonempty reason up to 16 KiB".into());
        }
    };
    serde_json::to_vec(&serde_json::json!({
        "kind":"action", "revision":revision, "action":action
    }))
    .map_err(|error| error.to_string())
}

pub(super) fn controls(
    state: &mut State,
    snapshot: &Value,
    current_evidence: bool,
    pending: bool,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let run_state = snapshot["report"]["state"].as_str().unwrap_or_default();
    let merge_running = snapshot["job"]["state"] == "running" && snapshot["job"]["kind"] == "merge";
    let has_candidate = snapshot["report"]["candidate"]["commit"].is_string();
    let can_approve = current_evidence && run_state == "waiting_approval";
    let can_merge = super::finalization::can_start_merge(snapshot, current_evidence, pending);
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
                    egui::Button::new("Approve candidate"),
                )
                .accessible_name("Approve verified candidate")
                .clicked()
            {
                state.confirming_decision = Some(CandidateDecision::Approve);
            }
            if ui
                .add_enabled(!pending, egui::Button::new("Reject candidate"))
                .accessible_name("Reject candidate")
                .clicked()
            {
                state.confirming_decision = Some(CandidateDecision::Reject);
            }
            if merge_running {
                let label = crate::ui::messages::text("room.merge_in_progress");
                ui.add_enabled(false, egui::Button::new(label))
                    .accessible_name(label);
            } else if ui
                .add_enabled(can_merge, egui::Button::new("Merge approved candidate"))
                .accessible_name("Merge approved candidate")
                .clicked()
            {
                state.confirming_decision = Some(CandidateDecision::Merge);
            }
        });
    }
    let decision = state.confirming_decision?;
    let label = match decision {
        CandidateDecision::Approve => "Confirm approval of this verified diff and tests?",
        CandidateDecision::Reject => "Reject this candidate and send feedback?",
        CandidateDecision::Merge => "Confirm integrating the exact approved candidate?",
    };
    ui.separator();
    ui.label(label);
    if decision == CandidateDecision::Reject {
        let label = ui.label("Rejection reason");
        ui.add_enabled(
            !pending,
            egui::TextEdit::singleline(&mut state.reject_reason),
        )
        .labelled_by(label.id)
        .accessible_name("Rejection reason");
    }
    let mut event = None;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(!pending, egui::Button::new("Cancel decision"))
            .accessible_name("Cancel candidate decision")
            .clicked()
        {
            state.confirming_decision = None;
        }
        let valid_reason = decision != CandidateDecision::Reject
            || !state.reject_reason.trim().is_empty() && state.reject_reason.len() <= 16 * 1024;
        if ui
            .add_enabled(
                !pending && valid_reason,
                egui::Button::new("Confirm decision"),
            )
            .accessible_name("Confirm candidate decision")
            .clicked()
        {
            event = Some(Event::Decision(decision, state.reject_reason.clone()));
            state.confirming_decision = None;
        }
    });
    event
}
