use super::super::Probe;
use super::State;

impl State {
    pub fn load_snapshot(&mut self, snapshot: &serde_json::Value) {
        let Ok(turns) =
            serde_json::from_value::<Vec<crate::chat::Turn>>(snapshot["chat_history"].clone())
        else {
            return;
        };
        if let Some(error) = self.uncertain_error.take() {
            let Some(pending) = self.pending_turn.take() else {
                self.failed(error);
                return;
            };
            let accepted = self.pending_before.as_ref().is_some_and(|before| {
                before.len() != turns.len()
                    || before
                        .iter()
                        .zip(&turns)
                        .any(|(old, new)| old.role != new.role || old.text != new.text)
            }) && turns
                .iter()
                .any(|turn| turn.role == pending.role && turn.text == pending.text);
            self.pending_before = None;
            self.requesting = false;
            self.turns = crate::chat::recent_history(&turns);
            if !accepted {
                self.job = None;
                self.draft = pending.text;
                self.error = Some(error);
                return;
            }
            self.error = None;
            if snapshot["job"]["kind"] == "chat"
                && snapshot["job"]["state"] == "running"
                && let Some(job) = snapshot["job"]["id"].as_str()
            {
                self.started(job.to_owned());
            } else if snapshot["job"]["kind"] == "chat" && snapshot["job"]["state"] == "failed" {
                self.failed(
                    snapshot["job"]["error"]
                        .as_str()
                        .unwrap_or("Coordinator could not answer")
                        .to_owned(),
                );
            }
            return;
        }
        if self.awaiting() {
            return;
        }
        self.turns = crate::chat::recent_history(&turns);
        if snapshot["job"]["kind"] == "chat" {
            if snapshot["job"]["state"] == "running"
                && let Some(job) = snapshot["job"]["id"].as_str()
            {
                self.started(job.to_owned());
            } else if snapshot["job"]["state"] == "failed" {
                self.failed(
                    snapshot["job"]["error"]
                        .as_str()
                        .unwrap_or("Coordinator could not answer")
                        .to_owned(),
                );
            }
        }
    }

    pub fn reset_pending(&mut self) {
        self.resetting = true;
    }

    pub fn reset_failed(&mut self) {
        self.resetting = false;
    }
}

impl Probe {
    pub(in crate::native_ui_probe::workspace) fn reset_coordinator_chat(&mut self) {
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Workspace revision is unavailable".to_owned())
            .and_then(|snapshot| {
                let revision = snapshot["revision"]
                    .as_str()
                    .ok_or_else(|| "Workspace revision is unavailable".to_owned())?;
                let packet = serde_json::to_vec(&serde_json::json!({
                    "kind":"chat_reset", "revision":revision
                }))
                .map_err(|error| error.to_string())?;
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                self.coordinator_chat.reset_pending();
                self.pending = Some(super::super::super::Pending::ChatReset);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }
}
