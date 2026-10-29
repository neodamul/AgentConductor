use super::super::Probe;
use super::State;

impl State {
    pub fn load_snapshot(&mut self, snapshot: &serde_json::Value) {
        if self.awaiting() {
            return;
        }
        if let Ok(turns) =
            serde_json::from_value::<Vec<crate::chat::Turn>>(snapshot["chat_history"].clone())
        {
            self.turns = crate::chat::recent_history(&turns);
        }
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
