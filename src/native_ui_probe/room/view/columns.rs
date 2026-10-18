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
    let width = ui.available_width();
    let summary_width = (width * 0.22).clamp(200.0, 270.0);
    let inspector_width = (width * 0.27).clamp(260.0, 330.0);
    // Reserve the two column gaps and the inspector's vertical scrollbar.
    let activity_width = (width - summary_width - inspector_width - 36.0).max(280.0);
    let height = ui.available_height();
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(summary_width, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(summary_width);
                egui::ScrollArea::vertical()
                    .id_salt("room-summary")
                    .show(ui, |ui| {
                        ui.set_width(summary_width);
                        super::super::super::render::snapshot(snapshot, language, ui);
                    });
            },
        );
        ui.allocate_ui_with_layout(
            egui::vec2(activity_width, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(activity_width);
                egui::ScrollArea::vertical()
                    .id_salt("room-activity")
                    .show(ui, |ui| {
                        ui.set_width(activity_width);
                        let index = state.task_controls.selected_index(snapshot).or_else(|| {
                            snapshot["report"]["plan"]["tasks"]
                                .as_array()
                                .and_then(|tasks| (!tasks.is_empty()).then_some(0))
                        });
                        if let Some(index) = index
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
                                .and_then(|agents| {
                                    agents.iter().find(|agent| agent["id"] == task["agent"])
                                })
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
                        if state.activity_tab == task_activity::Tab::Review {
                            let current = state
                                .evidence
                                .as_ref()
                                .filter(|evidence| evidence["revision"] == snapshot["revision"]);
                            if let Some(evidence) = current {
                                super::super::super::render::evidence_in_language(
                                    evidence,
                                    &mut state.diff,
                                    language,
                                    ui,
                                );
                            } else {
                                if state.evidence.is_some() {
                                    let warning = crate::ui::messages::in_language(
                                        language,
                                        "room.run_state_changed_review_the_latest_evidence_before",
                                    );
                                    ui.colored_label(ui.visuals().warn_fg_color, warning)
                                        .accessible_name(warning);
                                }
                                let (label, action) = if snapshot["report"]["candidate"].is_object()
                                {
                                    (
                                        if state.evidence.is_some() {
                                            "room.review_latest"
                                        } else {
                                            "room.load_diff_and_tests"
                                        },
                                        Some(Event::Evidence),
                                    )
                                } else {
                                    ("room.load_a_candidate_to_review_its_exact_evidence", None)
                                };
                                if ui
                                    .add_enabled(
                                        action.is_some() && !pending,
                                        egui::Button::new(crate::ui::messages::in_language(
                                            language, label,
                                        )),
                                    )
                                    .accessible_name(crate::ui::messages::in_language(
                                        language,
                                        if label == "room.load_diff_and_tests" {
                                            "room.load_review_evidence"
                                        } else {
                                            label
                                        },
                                    ))
                                    .clicked()
                                {
                                    event = action;
                                }
                            }
                        }
                    });
            },
        );
        ui.allocate_ui_with_layout(
            egui::vec2(inspector_width, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(inspector_width);
                egui::ScrollArea::vertical()
                    .id_salt("room-inspector")
                    .show(ui, |ui| {
                        ui.set_width(inspector_width);
                        if let Some(action) = task_controls::show(
                            &mut state.task_controls,
                            snapshot,
                            pending,
                            language,
                            ui,
                        ) {
                            event = Some(Event::TaskAction(action));
                        }
                        let current_evidence = state
                            .evidence
                            .as_ref()
                            .is_some_and(|evidence| evidence["revision"] == snapshot["revision"]);
                        if let Some(decision) = decision_controls(
                            state,
                            snapshot,
                            current_evidence,
                            pending,
                            language,
                            ui,
                        ) {
                            event = Some(decision);
                        }
                    });
            },
        );
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
            let width = ui.available_width();
            let response = ui
                .add_enabled_ui(!pending && enabled, |ui| {
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
