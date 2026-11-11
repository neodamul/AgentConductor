use super::ToolAction;
use eframe::egui;
use serde_json::Value;

pub fn tool_controls(
    state: &mut super::super::room::State,
    snapshot: Option<&Value>,
    pending: bool,
    ui: &mut egui::Ui,
) -> Option<super::super::room::Event> {
    let snapshot = snapshot?;
    let revision = snapshot["revision"].as_str().unwrap_or_default();
    let Some(requests) = snapshot["tool_requests"].as_array() else {
        state.confirming_tool = None;
        state.tool_dialog = None;
        return None;
    };
    let live = snapshot["report"]["state"] == "running";
    let can_decide = !pending && (live || snapshot["job"]["state"] != "running");
    let confirmation_is_current =
        state
            .confirming_tool
            .as_ref()
            .is_some_and(|(id, action, rev)| {
                rev == revision
                    && requests.iter().any(|request| {
                        request["id"].as_str() == Some(id.as_str())
                            && match action {
                                ToolAction::Approve | ToolAction::Reject => {
                                    request["status"] == "pending"
                                }
                                ToolAction::Refresh => matches!(
                                    request["status"].as_str(),
                                    Some("pending" | "approved")
                                ),
                            }
                    })
            });
    if !confirmation_is_current {
        state.confirming_tool = None;
    } else if let Some((id, _, _)) = state.confirming_tool.as_ref() {
        state.tool_dialog = Some(id.clone());
    }
    let selected_is_current = state.tool_dialog.as_ref().is_some_and(|id| {
        requests.iter().any(|request| {
            request["id"].as_str() == Some(id.as_str())
                && (request["status"] == "pending"
                    || (request["status"] == "approved"
                        && state
                            .confirming_tool
                            .as_ref()
                            .is_some_and(|(current, action, rev)| {
                                current == id && *action == ToolAction::Refresh && rev == revision
                            })))
        })
    });
    if state.tool_dialog.is_some() && !selected_is_current {
        state.tool_dialog = None;
    }
    if state.tool_dialog.is_none() {
        state.tool_dialog = requests
            .iter()
            .find(|request| {
                request["status"] == "pending"
                    && request["id"]
                        .as_str()
                        .is_some_and(|id| !state.deferred_tools.contains(id))
            })
            .and_then(|request| request["id"].as_str().map(str::to_owned));
    }
    if requests.is_empty() {
        return None;
    }
    ui.separator();
    ui.heading(format!("Tool requests · {}", requests.len()));
    egui::ScrollArea::vertical()
        .max_height(240.0)
        .show(ui, |ui| {
            for request in requests {
                ui.group(|ui| {
                    ui.label(format!(
                        "{} · {}",
                        request["request"]["tool"]
                            .as_str()
                            .unwrap_or("Unknown tool"),
                        request["status"].as_str().unwrap_or("unknown")
                    ));
                    ui.label(format!(
                        "Task · {}",
                        request["context"]["task"].as_str().unwrap_or("unknown")
                    ));
                    ui.label(
                        request["request"]["reason"]
                            .as_str()
                            .unwrap_or("No reason recorded"),
                    );
                    let detail = serde_json::to_string_pretty(&request["request"])
                        .unwrap_or_else(|_| "Tool input cannot be displayed".into());
                    egui::ScrollArea::vertical()
                        .max_height(120.0)
                        .show(ui, |ui| ui.monospace(detail));
                    if matches!(request["status"].as_str(), Some("pending" | "approved")) {
                        ui.horizontal(|ui| {
                            let actions = if request["status"] == "pending" {
                                vec![
                                    (ToolAction::Approve, "Allow once"),
                                    (ToolAction::Reject, "Deny"),
                                    (ToolAction::Refresh, "Refresh request"),
                                ]
                            } else {
                                vec![(ToolAction::Refresh, "Refresh request")]
                            };
                            for (action, label) in actions {
                                if ui
                                    .add_enabled(can_decide, egui::Button::new(label))
                                    .accessible_name(format!(
                                        "{label} tool request {}",
                                        request["id"].as_str().unwrap_or("unknown")
                                    ))
                                    .clicked()
                                    && let (Some(id), Some(revision)) =
                                        (request["id"].as_str(), snapshot["revision"].as_str())
                                {
                                    state.confirming_tool =
                                        Some((id.to_owned(), action, revision.to_owned()));
                                }
                            }
                        });
                    }
                });
            }
        });
    let id = state.tool_dialog.clone()?;
    let request = requests
        .iter()
        .find(|request| request["id"].as_str() == Some(id.as_str()))?
        .clone();
    let confirmation = state
        .confirming_tool
        .as_ref()
        .filter(|(current, _, rev)| current == &id && rev == revision)
        .map(|(_, action, _)| *action);
    let mut event = None;
    egui::Modal::new(egui::Id::unique("pending-tool-request")).show(ui.ctx(), |ui| {
        ui.set_max_width(560.0);
        if request["status"] == "approved" {
            ui.heading("Refresh approved tool request");
        } else {
            ui.heading("Tool request needs your decision");
        }
        ui.label(format!(
            "{} · task {}",
            request["request"]["tool"]
                .as_str()
                .unwrap_or("Unknown tool"),
            request["context"]["task"].as_str().unwrap_or("unknown")
        ));
        ui.label(
            request["request"]["reason"]
                .as_str()
                .unwrap_or("No reason recorded"),
        );
        let detail = serde_json::to_string_pretty(&request["request"])
            .unwrap_or_else(|_| "Tool input cannot be displayed".into());
        egui::ScrollArea::vertical()
            .max_height(220.0)
            .show(ui, |ui| ui.monospace(detail));
        if let Some(action) = confirmation {
            ui.label(match action {
                ToolAction::Approve => "Allow this exact tool input once?",
                ToolAction::Reject => "Deny this exact tool request?",
                ToolAction::Refresh => "Revoke this request and issue a fresh decision?",
            });
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!pending, egui::Button::new("Cancel tool decision"))
                    .accessible_name("Cancel tool request decision")
                    .clicked()
                {
                    state.confirming_tool = None;
                }
                if ui
                    .add_enabled(can_decide, egui::Button::new("Confirm tool decision"))
                    .accessible_name("Confirm tool request decision")
                    .clicked()
                {
                    event = Some(super::super::room::Event::ToolDecision(action, id.clone()));
                    state.confirming_tool = None;
                }
            });
        } else {
            ui.horizontal(|ui| {
                for (action, label) in [
                    (ToolAction::Approve, "Allow once"),
                    (ToolAction::Reject, "Deny"),
                ] {
                    if ui
                        .add_enabled(can_decide, egui::Button::new(label))
                        .accessible_name(format!("{label} tool request {id}"))
                        .clicked()
                    {
                        state.confirming_tool = Some((id.clone(), action, revision.to_owned()));
                    }
                }
                if ui.button("Decide later").clicked() {
                    state.deferred_tools.insert(id.clone());
                    state.tool_dialog = None;
                }
            });
        }
    });
    event
}
