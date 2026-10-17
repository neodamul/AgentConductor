use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Feedback { task: String, feedback: String },
    Reassign { task: String, agent: String },
    SetModel { task: String, model: String },
    Schedule { task: String, at: String },
}

#[derive(Default)]
pub struct State {
    selected_task: String,
    target_agent: String,
    assigned_agent: String,
    selected_model: String,
    model_task: String,
    model_agent: String,
    assigned_model: String,
    feedback: String,
    feedback_task: String,
    feedback_by_task: HashMap<String, String>,
    schedule_at: String,
    schedule_task: String,
    schedule_by_task: HashMap<String, String>,
}

impl State {
    pub fn selected_index(&self, snapshot: &Value) -> Option<usize> {
        snapshot["report"]["plan"]["tasks"]
            .as_array()?
            .iter()
            .position(|task| task["id"].as_str() == Some(self.selected_task.as_str()))
    }
}

mod request;
mod view;
pub use request::request;
pub use view::show;
