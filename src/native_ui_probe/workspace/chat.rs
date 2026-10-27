mod request;
mod view;
pub use request::request;

pub enum Event {
    Send(String),
    Reset,
}

#[derive(Default)]
pub struct State {
    open: bool,
    draft: String,
    turns: Vec<crate::chat::Turn>,
    requesting: bool,
    job: Option<String>,
    error: Option<String>,
    editor_focused: bool,
    workspace: Option<String>,
}

impl State {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn awaiting(&self) -> bool {
        self.requesting || self.job.is_some()
    }

    pub fn draft(&self) -> &str {
        &self.draft
    }

    pub fn transcript(&self) -> &[crate::chat::Turn] {
        &self.turns
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn restore(&mut self, snapshot: &serde_json::Value) {
        let workspace = snapshot["config"]["repository"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        if self
            .workspace
            .as_ref()
            .is_some_and(|current| current != &workspace)
        {
            self.turns.clear();
            self.draft.clear();
            self.job = None;
        }
        self.workspace = Some(workspace);
        if !self.requesting
            && snapshot["chat_transcript"]
                .as_array()
                .is_some_and(|turns| turns.len() >= self.turns.len())
            && let Ok(turns) = serde_json::from_value(snapshot["chat_transcript"].clone())
        {
            self.turns = turns;
        }
        if snapshot["job"]["kind"] == "chat" && snapshot["job"]["state"] == "running" {
            self.job = snapshot["job"]["id"].as_str().map(str::to_owned);
            self.requesting = false;
        }
    }

    pub fn started(&mut self, job: String) {
        self.requesting = false;
        self.job = Some(job);
        self.error = None;
    }

    pub fn completed(&mut self, job: &str, answer: String) -> bool {
        if self.job.as_deref() != Some(job) || answer.trim().is_empty() {
            return false;
        }
        self.turns.push(crate::chat::Turn {
            role: "assistant".into(),
            text: answer,
        });
        self.job = None;
        self.requesting = false;
        true
    }

    pub fn failed(&mut self, message: String) {
        self.job = None;
        self.requesting = false;
        self.error = Some(message);
    }

    pub fn reset_completed(&mut self) {
        self.requesting = false;
        self.error = None;
    }
}

impl super::super::Probe {
    pub(super) fn reset_coordinator_chat(&mut self) {
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
            Ok(()) => self.pending = Some(super::super::Pending::ChatReset),
            Err(error) => self.coordinator_chat.failed(error),
        }
    }
    pub(super) fn send_coordinator_chat(&mut self, question: String) {
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
                self.pending = Some(super::super::Pending::ChatStart);
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
            Ok(()) => self.pending = Some(super::super::Pending::ChatRefresh),
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
        let Some(job) = self.coordinator_chat.job.clone() else {
            self.pending = None;
            return;
        };
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
