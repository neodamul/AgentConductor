use super::super::{Event, State, task_activity, task_controls};
use eframe::egui::{self, Ui};
use serde_json::Value;
mod decision;

pub(super) fn show(
    state: &mut State,
    snapshot: &Value,
    pending: bool,
    request_details: bool,
    language: &str,
    ui: &mut Ui,
) -> Option<Event> {
    let mut event = None;
    state.inspector.observe(snapshot);
    let width = ui.available_width();
    let summary_width = (width * 0.18).clamp(180.0, 240.0);
    let inspector_width = if !state.inspector.open {
        56.0
    } else if width >= 1000.0 {
        (width * 0.24).clamp(280.0, 320.0)
    } else {
        (width * 0.27).clamp(260.0, 330.0)
    };
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
                        let selected = state
                            .task_controls
                            .selected_index(snapshot)
                            .and_then(|index| {
                                snapshot["report"]["plan"]["tasks"][index]["id"].as_str()
                            })
                            .or_else(|| snapshot["report"]["plan"]["tasks"][0]["id"].as_str());
                        let selected = if state.coordinator_selected {
                            Some("")
                        } else {
                            selected
                        };
                        if let Some(task) = super::super::super::render::snapshot_selectable(
                            snapshot, language, selected, false, ui,
                        ) {
                            state.task_controls.select_task(&task);
                            state.coordinator_selected = false;
                        }
                        let label = crate::ui::messages::interpolate(
                            crate::ui::messages::in_language(language, "room.decisions_count"),
                            &[(
                                "value1",
                                &snapshot["decision_count"].as_u64().unwrap_or(0).to_string(),
                            )],
                        );
                        if ui
                            .add_enabled(
                                !pending,
                                egui::Button::new(&label)
                                    .selected(state.coordinator_selected)
                                    .wrap(),
                            )
                            .accessible_name(label)
                            .clicked()
                        {
                            state.coordinator_selected = true;
                            event = Some(Event::Coordinator);
                        }
                        super::verification::show(snapshot, language, ui);
                        if let Some(action) =
                            super::human_review::show(state, snapshot, pending, language, ui)
                        {
                            event = Some(action);
                        }
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
                        if state.coordinator_selected {
                            if state.coordinator_judgments.is_none()
                                && request_details
                                && !pending
                                && event.is_none()
                            {
                                event = Some(Event::Coordinator);
                            }
                            if let Some(action) = task_activity::coordinator(
                                &mut state.activity_tab,
                                state.coordinator_judgments.as_ref(),
                                language,
                                ui,
                            ) {
                                event = Some(action);
                            }
                        } else if let Some(index) = index
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
                                provider,
                                task,
                                state.task_details.as_ref(),
                                snapshot["activity"].get(task_id),
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
                state.inspector.heading(language, ui);
                if !state.inspector.open {
                    return;
                }
                egui::ScrollArea::vertical()
                    .id_salt("room-inspector")
                    .show(ui, |ui| {
                        ui.set_width(inspector_width);
                        let current_evidence = state
                            .evidence
                            .as_ref()
                            .is_some_and(|evidence| evidence["revision"] == snapshot["revision"]);
                        if let Some(decision) =
                            decision::show(state, snapshot, current_evidence, pending, language, ui)
                        {
                            event = Some(decision);
                        }
                        if !state.coordinator_selected
                            && let Some(action) = task_controls::show(
                                &mut state.task_controls,
                                snapshot,
                                pending,
                                language,
                                ui,
                            )
                        {
                            event = Some(Event::TaskAction(action));
                        }
                        if state.coordinator_selected
                            && let Some(action) = super::super::coordinator_feedback::show(
                                &mut state.task_controls.coordinator_feedback,
                                state.coordinator_judgments.as_ref(),
                                pending,
                                language,
                                ui,
                            )
                        {
                            event = Some(Event::TaskAction(action));
                        }
                    });
            },
        );
    });
    event
}
