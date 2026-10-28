mod controller;
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
    pending_question: Option<String>,
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

    pub fn begin_request(&mut self, question: String) {
        self.pending_question = Some(question.clone());
        self.turns.push(crate::chat::Turn {
            role: "operator".into(),
            text: question,
        });
        self.requesting = true;
        self.error = None;
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
            self.pending_question = None;
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
        self.pending_question = None;
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
        self.pending_question = None;
        self.requesting = false;
        true
    }

    pub fn reconnect(&mut self, snapshot: &serde_json::Value) -> bool {
        if self.restore_persisted_answer(snapshot) {
            return true;
        }
        if snapshot["job"]["kind"] != "chat" {
            return false;
        }
        if snapshot["job"]["state"] == "running"
            && let Some(job) = snapshot["job"]["id"].as_str()
        {
            self.started(job.to_owned());
            return true;
        }
        if snapshot["job"]["state"] == "completed"
            && let Ok(turns) = serde_json::from_value(snapshot["chat_transcript"].clone())
        {
            self.turns = turns;
            self.pending_question = None;
            self.job = None;
            self.requesting = false;
            return true;
        }
        false
    }

    fn restore_persisted_answer(&mut self, snapshot: &serde_json::Value) -> bool {
        if snapshot["job"].is_object() {
            return false;
        }
        let Ok(turns) =
            serde_json::from_value::<Vec<crate::chat::Turn>>(snapshot["chat_transcript"].clone())
        else {
            return false;
        };
        let Some(question) = self.turns.iter().rev().find(|turn| turn.role == "operator") else {
            return false;
        };
        let has_answer = turns
            .iter()
            .position(|turn| turn.role == "operator" && turn.text == question.text)
            .is_some_and(|index| {
                turns[index + 1..]
                    .iter()
                    .any(|turn| turn.role == "assistant" && !turn.text.trim().is_empty())
            });
        if !has_answer {
            return false;
        }
        self.turns = turns;
        self.job = None;
        self.pending_question = None;
        self.requesting = false;
        true
    }

    pub fn request_failed(&mut self, message: String) {
        self.error = Some(message);
    }

    pub fn failed(&mut self, message: String) {
        if self.job.is_none()
            && let Some(question) = self.pending_question.take()
        {
            if self
                .turns
                .last()
                .is_some_and(|turn| turn.role == "operator" && turn.text == question)
            {
                self.turns.pop();
            }
            self.draft = question;
        }
        self.job = None;
        self.requesting = false;
        self.error = Some(message);
    }

    pub fn reset_completed(&mut self) {
        self.requesting = false;
        self.error = None;
    }
}
