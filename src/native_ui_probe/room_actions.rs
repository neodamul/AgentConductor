use super::{Pending, Probe};
use eframe::egui;
use serde_json::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunAction {
    Resume,
    Recover,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolAction {
    Approve,
    Reject,
    Refresh,
}

pub fn tool_action_request(
    snapshot: &Value,
    id: &str,
    action: ToolAction,
) -> Result<Vec<u8>, String> {
    if id.len() != 64
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("invalid tool request ID".into());
    }
    let revision = snapshot["revision"]
        .as_str()
        .ok_or("current run revision is unavailable")?;
    let request = snapshot["tool_requests"]
        .as_array()
        .and_then(|requests| requests.iter().find(|request| request["id"] == id))
        .filter(|request| match action {
            ToolAction::Approve | ToolAction::Reject => request["status"] == "pending",
            ToolAction::Refresh => {
                matches!(request["status"].as_str(), Some("pending" | "approved"))
            }
        })
        .ok_or("tool request is no longer eligible for this action")?;
    if !request["request"].is_object() || !request["context"].is_object() {
        return Err("tool request evidence is incomplete".into());
    }
    let kind = match action {
        ToolAction::Approve => "tool_approve",
        ToolAction::Reject => "tool_reject",
        ToolAction::Refresh => "tool_refresh",
    };
    serde_json::to_vec(&serde_json::json!({
        "kind":"action", "revision":revision, "action":{"kind":kind,"sha":id}
    }))
    .map_err(|error| error.to_string())
}

pub(super) fn tool_controls(
    state: &mut super::room::State,
    snapshot: Option<&Value>,
    pending: bool,
    ui: &mut egui::Ui,
) -> Option<super::room::Event> {
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
    }
    if state.tool_dialog.as_ref().is_some_and(|id| {
        !requests.iter().any(|request| {
            request["id"].as_str() == Some(id.as_str()) && request["status"] == "pending"
        })
    }) {
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
    let mut event = None;
    if let Some((id, action, _)) = state
        .confirming_tool
        .clone()
        .filter(|(id, _, _)| state.tool_dialog.as_deref() != Some(id.as_str()))
        && let Some(request) = requests
            .iter()
            .find(|request| request["id"].as_str() == Some(id.as_str()))
    {
        ui.label(match action {
            ToolAction::Approve => "Allow this exact tool input once?",
            ToolAction::Reject => "Deny this exact tool request?",
            ToolAction::Refresh => "Revoke this request and issue a fresh decision?",
        });
        ui.monospace(format!(
            "{} · {id}",
            request["request"]["tool"]
                .as_str()
                .unwrap_or("Unknown tool")
        ));
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
                event = Some(super::room::Event::ToolDecision(action, id.clone()));
                state.confirming_tool = None;
            }
        });
    }
    if let Some(id) = state.tool_dialog.clone()
        && let Some(request) = requests
            .iter()
            .find(|request| request["id"].as_str() == Some(id.as_str()))
            .cloned()
    {
        let confirmation = state
            .confirming_tool
            .as_ref()
            .filter(|(current, _, rev)| current == &id && rev == revision)
            .map(|(_, action, _)| *action);
        let mut dialog_event = None;
        egui::Modal::new(egui::Id::unique("pending-tool-request")).show(ui.ctx(), |ui| {
            ui.set_max_width(560.0);
            ui.heading("Tool request needs your decision");
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
                    ToolAction::Refresh => "Refresh this request?",
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
                        dialog_event = Some(super::room::Event::ToolDecision(action, id.clone()));
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
        if dialog_event.is_some() {
            event = dialog_event;
        }
    }
    event
}

pub(super) fn controls(
    state: &mut super::room::State,
    snapshot: Option<&Value>,
    pending: bool,
    ui: &mut egui::Ui,
) -> Option<super::room::Event> {
    let snapshot = snapshot?;
    let run_state = snapshot["report"]["state"].as_str().unwrap_or_default();
    let busy = pending || snapshot["job"]["state"] == "running";
    if run_state != "running" {
        state.confirming_recover = false;
    }
    if !state.confirming_recover {
        let mut event = None;
        ui.horizontal(|ui| {
            let resumable = !matches!(
                run_state,
                "completed" | "approved" | "waiting_approval" | "rejected"
            );
            if ui
                .add_enabled(!busy && resumable, egui::Button::new("Resume run"))
                .accessible_name("Resume run")
                .clicked()
            {
                event = Some(super::room::Event::Resume);
            }
            if ui
                .add_enabled(
                    !busy && run_state == "running",
                    egui::Button::new("Recover run"),
                )
                .accessible_name("Recover interrupted run")
                .clicked()
            {
                state.confirming_recover = true;
            }
        });
        return event;
    }
    ui.separator();
    ui.label("Recovery refuses any live or unknown recorded process.");
    let mut event = None;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(!busy, egui::Button::new("Cancel recovery"))
            .accessible_name("Cancel recovery")
            .clicked()
        {
            state.confirming_recover = false;
        }
        if ui
            .add_enabled(!busy, egui::Button::new("Confirm recovery"))
            .accessible_name("Confirm run recovery")
            .clicked()
        {
            state.confirming_recover = false;
            event = Some(super::room::Event::Recover);
        }
    });
    event
}

pub fn run_action_request(snapshot: &Value, action: RunAction) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .ok_or("current run revision is unavailable")?;
    let state = snapshot["report"]["state"]
        .as_str()
        .ok_or("current run state is unavailable")?;
    let kind = match action {
        RunAction::Resume
            if !matches!(
                state,
                "completed" | "approved" | "waiting_approval" | "rejected"
            ) =>
        {
            "resume"
        }
        RunAction::Recover if state == "running" => "recover",
        RunAction::Resume => return Err("this run cannot be resumed in its current state".into()),
        RunAction::Recover => return Err("only a running run can be recovered".into()),
    };
    serde_json::to_vec(&serde_json::json!({
        "kind":"action", "revision":revision, "action":{"kind":kind}
    }))
    .map_err(|error| error.to_string())
}

impl Probe {
    pub(super) fn resume_room(&mut self) {
        self.run_action(RunAction::Resume);
    }

    pub(super) fn recover_room(&mut self) {
        self.run_action(RunAction::Recover);
    }

    pub(super) fn decide_tool(&mut self, action: ToolAction, id: String) {
        let resume_after = action == ToolAction::Approve
            && self
                .snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot["report"]["state"] != "running");
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Agent Room snapshot is unavailable".to_owned())
            .and_then(|snapshot| tool_action_request(snapshot, &id, action))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Agent Room controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(Pending::ToolDecision { resume_after });
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn run_action(&mut self, action: RunAction) {
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Agent Room snapshot is unavailable".to_owned())
            .and_then(|snapshot| run_action_request(snapshot, action))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Agent Room controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(Pending::Action);
            }
            Err(error) => self.error = Some(error),
        }
    }
}
