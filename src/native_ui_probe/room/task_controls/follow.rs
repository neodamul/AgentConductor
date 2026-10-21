use serde_json::Value;

impl super::State {
    pub fn follow_progress(&mut self, snapshot: &Value) {
        let Some(tasks) = snapshot["report"]["plan"]["tasks"].as_array() else {
            return;
        };
        let Some(first) = tasks.first().and_then(|task| task["id"].as_str()) else {
            return;
        };
        if !tasks.iter().any(|task| task["id"] == self.selected_task) {
            self.selected_task = first.into();
        }
        let status = |id: &str| {
            snapshot["report"]["tasks"]
                .as_array()?
                .iter()
                .find(|task| task["id"] == id)?["status"]
                .as_str()
        };
        let selected = self.selected_task.clone();
        let finished = self
            .last_observed_status
            .as_ref()
            .is_some_and(|(id, previous)| id == &selected && previous == "running")
            && status(&selected).is_some_and(|current| current != "running");
        if finished
            && let Some(next) = snapshot["report"]["tasks"].as_array().and_then(|reports| {
                reports.iter().find(|task| {
                    task["id"] != selected
                        && matches!(
                            task["status"].as_str(),
                            Some("running" | "waiting_tool_approval")
                        )
                })
            })
            && let Some(next) = next["id"].as_str()
        {
            self.select_task(next);
        }
        self.last_observed_status =
            status(&self.selected_task).map(|current| (self.selected_task.clone(), current.into()));
    }
}
