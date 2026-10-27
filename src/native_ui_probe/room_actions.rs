use super::{Pending, Probe};
use eframe::egui;
use serde_json::Value;

mod task;
mod tool_view;
pub(super) use tool_view::tool_controls;

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

pub(super) fn controls(
    state: &mut super::room::State,
    snapshot: Option<&Value>,
    pending: bool,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<super::room::Event> {
    let text = |id| crate::ui::messages::in_language(language, id);
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
                .add_enabled(
                    !busy && resumable,
                    egui::Button::new(text("room.resume_run")),
                )
                .accessible_name(text("room.resume_run"))
                .clicked()
            {
                event = Some(super::room::Event::Resume);
            }
            if ui
                .add_enabled(
                    !busy && run_state == "running",
                    egui::Button::new(text("room.recover_interrupted_run")),
                )
                .accessible_name(text("room.recover_interrupted_run"))
                .clicked()
            {
                state.confirming_recover = true;
            }
        });
        return event;
    }
    ui.separator();
    ui.label(text(
        "room.recovery_refuses_any_live_or_unknown_recorded_process",
    ));
    let mut event = None;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(!busy, egui::Button::new(text("room.cancel_recovery")))
            .accessible_name(text("room.cancel_recovery"))
            .clicked()
        {
            state.confirming_recover = false;
        }
        if ui
            .add_enabled(!busy, egui::Button::new(text("room.confirm_recovery")))
            .accessible_name(text("room.confirm_recovery"))
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
