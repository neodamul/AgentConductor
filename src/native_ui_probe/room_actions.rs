use super::{Pending, Probe};
use eframe::egui;
use serde_json::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunAction {
    Resume,
    Recover,
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
