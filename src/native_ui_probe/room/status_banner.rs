use eframe::egui::{self, RichText};
use serde_json::Value;

pub(super) fn show(
    room: &mut super::State,
    snapshot: &Value,
    pending: bool,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<super::Event> {
    let text = |key| crate::ui::messages::in_language(language, key);
    let report = &snapshot["report"];
    let state = report["state"].as_str().unwrap_or("unknown");
    let merge_running = snapshot["job"]["kind"] == "merge" && snapshot["job"]["state"] == "running";
    let tasks = report["tasks"].as_array();
    let total = report["plan"]["tasks"].as_array().map_or(0, Vec::len);
    let completed = tasks.map_or(0, |items| {
        items
            .iter()
            .filter(|task| task["status"] == "completed")
            .count()
    });
    let error = report["error"]
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            tasks.and_then(|items| {
                items.iter().find(|task| task["status"] == "failed")?["error"].as_str()
            })
        });
    let (foreground, background) =
        super::super::render::status_colors(state, ui.visuals().dark_mode);
    let label = super::super::render::status_label(state, language);
    let mut event = None;
    egui::Frame::new()
        .fill(background)
        .stroke(egui::Stroke::new(1.0, foreground))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(text("room.run_status"))
                        .small()
                        .strong()
                        .color(foreground),
                );
                ui.label(RichText::new(format!("● {}", label)).strong().color(
                    super::super::render::status_color_with_motion(
                        state,
                        ui.input(|input| input.time),
                        ui.style().animation_time > 0.0,
                    ),
                ));
                ui.label(crate::ui::messages::interpolate(
                    text("room.tasks_completed"),
                    &[
                        ("value1", &completed.to_string()),
                        ("value2", &total.to_string()),
                    ],
                ));
            });
            if merge_running {
                ui.label(RichText::new(text("room.merge_in_progress")).strong());
                ui.label(text("room.applying_approved_candidate_to_workspace"));
            }
            if let Some(error) = error {
                ui.colored_label(
                    super::super::render::status_color("failed", ui.input(|input| input.time)),
                    format!("{}{}", text("room.last_error"), error),
                );
            }
            if state == "failed"
                && controls_can_draft(snapshot)
                && ui
                    .button(text("room.fix_with_feedback"))
                    .accessible_name(text("room.fix_with_feedback"))
                    .clicked()
            {
                room.task_controls.draft_failed_task_feedback(snapshot);
            }
            if let Some((description, action)) = next_step(room, snapshot) {
                ui.separator();
                ui.label(RichText::new(text("room.next_step")).small().strong());
                ui.label(text(description));
                let control = match action {
                    NextAction::Review => Some(("room.review_changes", None)),
                    NextAction::Approve => Some((
                        "room.next_approve_action",
                        Some(super::CandidateDecision::Approve),
                    )),
                    NextAction::Merge => Some((
                        if merge_running {
                            "room.merge_in_progress"
                        } else {
                            "room.next_merge_action"
                        },
                        Some(super::CandidateDecision::Merge),
                    )),
                    NextAction::Workspace => Some(("room.next_workspace_action", None)),
                    NextAction::WorkspaceUnavailable => None,
                };
                if let Some((label, confirmation)) = control
                    && ui
                        .add_enabled(
                            !pending && snapshot["job"]["state"] != "running",
                            egui::Button::new(text(label)),
                        )
                        .accessible_name(text(label))
                        .clicked()
                {
                    if let Some(decision) = confirmation {
                        room.confirming_decision = Some(decision);
                    } else if action == NextAction::Workspace {
                        event = Some(super::Event::ReturnToWorkspace);
                    } else {
                        room.activity_tab = super::task_activity::Tab::Review;
                        event = Some(super::Event::Evidence);
                    }
                }
            }
        });
    event
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum NextAction {
    Review,
    Approve,
    Merge,
    Workspace,
    WorkspaceUnavailable,
}

fn next_step(room: &super::State, snapshot: &Value) -> Option<(&'static str, NextAction)> {
    if snapshot["report"]["state"] == "completed" {
        return Some(if room.workspace_available {
            ("room.next_completed", NextAction::Workspace)
        } else {
            (
                "room.next_workspace_unavailable",
                NextAction::WorkspaceUnavailable,
            )
        });
    }
    let candidate = snapshot["report"]["candidate"]["commit"].as_str()?;
    if candidate.is_empty() {
        return None;
    }
    let evidence_current = room
        .evidence
        .as_ref()
        .is_some_and(|evidence| evidence["revision"] == snapshot["revision"]);
    match snapshot["report"]["state"].as_str()? {
        "waiting_approval" if evidence_current => {
            Some(("room.next_approve_candidate", NextAction::Approve))
        }
        "waiting_approval" => Some(("room.next_review_candidate", NextAction::Review)),
        "approved" if evidence_current => Some(("room.next_merge_candidate", NextAction::Merge)),
        "approved" => Some(("room.next_review_candidate", NextAction::Review)),
        _ => None,
    }
}

fn controls_can_draft(snapshot: &Value) -> bool {
    snapshot["report"]["plan"]["tasks"]
        .as_array()
        .is_some_and(|tasks| {
            tasks.iter().any(|task| {
                task["id"].as_str().is_some_and(|id| {
                    snapshot["report"]["tests"].as_array().is_some_and(|tests| {
                        tests.iter().any(|test| {
                            test["task"] == id
                                && (test["exit_code"].as_i64().is_some_and(|code| code != 0)
                                    || test["timed_out"] == true)
                        })
                    }) || snapshot["report"]["tasks"]
                        .as_array()
                        .is_some_and(|reports| {
                            reports
                                .iter()
                                .any(|report| report["id"] == id && report["status"] == "failed")
                        })
                })
            })
        })
}
