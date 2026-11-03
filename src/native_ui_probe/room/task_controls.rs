use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Feedback { task: String, feedback: String },
    CoordinatorRequest { message: String },
    Reassign { task: String, agent: String },
    SetModel { task: String, model: String },
    Schedule { task: String, at: String },
}

#[derive(Default)]
pub struct State {
    selected_task: String,
    selection_changed_from: Option<String>,
    last_observed_status: Option<(String, String)>,
    target_agent: String,
    assigned_agent: String,
    selected_model: String,
    model_task: String,
    model_agent: String,
    assigned_model: String,
    feedback: String,
    feedback_task: String,
    feedback_by_task: HashMap<String, String>,
    pub coordinator_feedback: String,
    schedule_at: String,
    schedule_task: String,
    schedule_by_task: HashMap<String, String>,
}

impl State {
    pub fn select_task(&mut self, id: &str) {
        if self.selected_task != id {
            self.selection_changed_from =
                Some(std::mem::replace(&mut self.selected_task, id.to_owned()));
        }
    }

    pub fn selected_index(&self, snapshot: &Value) -> Option<usize> {
        snapshot["report"]["plan"]["tasks"]
            .as_array()?
            .iter()
            .position(|task| task["id"].as_str() == Some(self.selected_task.as_str()))
    }

    pub fn draft_failed_task_feedback(&mut self, snapshot: &Value, language: &str) -> bool {
        if snapshot["report"]["state"] != "failed" {
            return false;
        }
        let Some(tasks) = snapshot["report"]["plan"]["tasks"].as_array() else {
            return false;
        };
        let failed = snapshot["report"]["tests"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|test| {
                test["exit_code"].as_i64().is_some_and(|code| code != 0)
                    || test["timed_out"] == true
            })
            .and_then(|test| {
                let id = test["task"].as_str()?;
                tasks.iter().find(|task| task["id"] == id)?;
                Some((
                    id,
                    test["output"]
                        .as_str()
                        .filter(|text| !text.trim().is_empty())
                        .or_else(|| snapshot["report"]["error"].as_str())
                        .unwrap_or(crate::ui::messages::in_language(
                            language,
                            "room.verification_failed_fallback",
                        )),
                ))
            })
            .or_else(|| {
                let failed = snapshot["report"]["tasks"]
                    .as_array()?
                    .iter()
                    .find(|task| task["status"] == "failed")?;
                let id = failed["id"].as_str()?;
                tasks.iter().find(|task| task["id"] == id)?;
                Some((
                    id,
                    failed["error"]
                        .as_str()
                        .or_else(|| snapshot["report"]["error"].as_str())
                        .unwrap_or(crate::ui::messages::in_language(
                            language,
                            "room.task_failed_fallback",
                        )),
                ))
            });
        let Some((id, cause)) = failed else {
            return false;
        };
        let cause = bounded_feedback(cause, 8 * 1024);
        if !self.feedback_task.is_empty() {
            self.feedback_by_task.insert(
                self.feedback_task.clone(),
                std::mem::take(&mut self.feedback),
            );
        }
        self.selected_task = id.to_owned();
        self.feedback_task = id.to_owned();
        let previous = self.feedback_by_task.remove(id).unwrap_or_default();
        let previous = bounded_feedback(&previous, 4 * 1024);
        let prefix = if previous.trim().is_empty() {
            String::new()
        } else {
            format!("{previous}\n\n")
        };
        let guidance = crate::ui::messages::interpolate(
            crate::ui::messages::in_language(language, "room.failed_task_feedback"),
            &[("value1", id), ("value2", &cause)],
        );
        self.feedback = bounded_feedback(&(prefix + &guidance), 16 * 1024);
        true
    }
}

fn bounded_feedback(value: &str, max_bytes: usize) -> String {
    value
        .chars()
        .scan(0, |bytes, ch| {
            *bytes += ch.len_utf8();
            (*bytes <= max_bytes).then_some(ch)
        })
        .collect()
}

mod follow;
mod request;
mod view;
pub use request::request;
pub use view::show;
