use super::request;

impl super::super::super::Probe {
    pub(in crate::native_ui_probe::workspace) fn reset_coordinator_chat(&mut self) {
        let Some(revision) = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot["revision"].as_str())
        else {
            return;
        };
        let packet =
            serde_json::to_vec(&serde_json::json!({"kind":"chat_reset","revision":revision}));
        let result = packet.map_err(|error| error.to_string()).and_then(|bytes| {
            self.worker
                .as_ref()
                .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                .send(&bytes)
        });
        match result {
            Ok(()) => self.pending = Some(super::super::super::Pending::ChatReset),
            Err(error) => self.coordinator_chat.failed(error),
        }
    }

    pub(in crate::native_ui_probe::workspace) fn send_coordinator_chat(
        &mut self,
        question: String,
    ) {
        let packet = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Workspace revision is unavailable".to_owned())
            .and_then(|snapshot| request(snapshot, &question));
        let result = packet.and_then(|packet| {
            self.worker
                .as_ref()
                .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                .send(&packet)
        });
        match result {
            Ok(()) => {
                self.pending = Some(super::super::super::Pending::ChatStart);
                self.error = None;
            }
            Err(error) => {
                self.coordinator_chat.failed(error);
                self.workspace_next_refresh = None;
            }
        }
    }

    pub(in crate::native_ui_probe) fn refresh_coordinator_chat(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"snapshot","chat_result":true}"#));
        match result {
            Ok(()) => self.pending = Some(super::super::super::Pending::ChatRefresh),
            Err(error) => {
                self.coordinator_chat.failed(error);
                self.workspace_next_refresh = None;
            }
        }
    }

    pub(in crate::native_ui_probe) fn chat_started(&mut self, snapshot: serde_json::Value) {
        if snapshot["accepted"] == true
            && let Some(job) = snapshot["id"].as_str()
        {
            self.coordinator_chat.started(job.to_owned());
            self.workspace_next_refresh =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(500));
            self.pending = None;
        } else {
            self.coordinator_chat
                .failed("Coordinator chat job was not accepted".into());
            self.pending = None;
        }
    }

    pub(in crate::native_ui_probe) fn chat_refreshed(&mut self, snapshot: serde_json::Value) {
        if self.coordinator_chat.job.is_some() && self.coordinator_chat.reconnect(&snapshot) {
            self.snapshot = Some(snapshot);
            self.workspace_next_refresh = None;
            self.pending = None;
            self.error = None;
            return;
        }
        if self.coordinator_chat.job.is_none() {
            if self.coordinator_chat.reconnect(&snapshot) {
                self.snapshot = Some(snapshot);
                self.workspace_next_refresh = if self.coordinator_chat.awaiting() {
                    Some(std::time::Instant::now() + std::time::Duration::from_millis(500))
                } else {
                    None
                };
                self.pending = None;
                self.error = None;
            } else {
                self.coordinator_chat
                    .failed("Coordinator chat request was not accepted".into());
                self.snapshot = Some(snapshot);
                self.workspace_next_refresh = None;
                self.pending = None;
            }
            return;
        }
        let job = self.coordinator_chat.job.clone().unwrap();
        if snapshot["job"]["id"] != job || snapshot["job"]["kind"] != "chat" {
            self.coordinator_chat
                .failed("Coordinator chat job no longer matches the request".into());
        } else if snapshot["job"]["state"] == "running" {
            self.snapshot = Some(snapshot);
            self.workspace_next_refresh =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(500));
        } else if snapshot["job"]["state"] == "completed" {
            let answer = snapshot["chat"]["answer"].as_str().unwrap_or_default();
            if !self.coordinator_chat.completed(&job, answer.to_owned()) {
                self.coordinator_chat
                    .failed("Coordinator chat returned no answer".into());
            }
            self.snapshot = Some(snapshot);
            self.workspace_next_refresh = None;
        } else {
            let error = snapshot["job"]["error"]
                .as_str()
                .filter(|text| !text.is_empty())
                .unwrap_or("Coordinator could not answer");
            self.coordinator_chat.failed(error.to_owned());
            self.snapshot = Some(snapshot);
            self.workspace_next_refresh = None;
        }
        self.pending = None;
        self.error = None;
    }
}
