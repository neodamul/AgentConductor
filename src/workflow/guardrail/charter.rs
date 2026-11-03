use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskContract {
    pub id: String,
    pub paths: Vec<String>,
    pub depends_on: Vec<String>,
    pub test_command: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Charter {
    pub goal: String,
    pub plan_rules_sha256: String,
    pub tasks: Vec<TaskContract>,
}

impl Charter {
    pub fn from_plan(plan: &Plan) -> Self {
        Self {
            goal: plan.goal.clone(),
            plan_rules_sha256: crate::workflow::digest(plan.plan_rules.as_bytes()),
            tasks: plan.tasks.iter().map(TaskContract::from).collect(),
        }
    }

    pub fn update_task(&mut self, plan: &Plan, index: usize) -> Result<(), String> {
        let task = plan.tasks.get(index).ok_or("unknown charter task")?;
        let contract = self
            .tasks
            .iter_mut()
            .find(|contract| contract.id == task.id)
            .ok_or("task is not present in the charter")?;
        *contract = task.into();
        Ok(())
    }
}

impl From<&crate::plan::Task> for TaskContract {
    fn from(task: &crate::plan::Task) -> Self {
        Self {
            id: task.id.clone(),
            paths: task.paths.clone(),
            depends_on: task.depends_on.clone(),
            test_command: task.test_command.clone(),
        }
    }
}

pub fn scope_growth(charter: &Charter, plan: &Plan) -> usize {
    let mut added = Vec::new();
    for task in &plan.tasks {
        let original = charter.tasks.iter().find(|item| item.id == task.id);
        for path in &task.paths {
            if !original.is_some_and(|item| item.paths.contains(path))
                && !added.contains(&(task.id.as_str(), path.as_str()))
            {
                added.push((task.id.as_str(), path.as_str()));
            }
        }
    }
    added.len()
}

pub fn charter_drift(plan: &Plan, charter: Option<&Charter>) -> Vec<Violation> {
    let Some(charter) = charter else {
        return Vec::new();
    };
    if Charter::from_plan(plan) == *charter {
        return Vec::new();
    }
    vec![Violation {
        code: "goal_drift",
        task: None,
        detail: "the goal, planning rules, or task contract changed without a recorded amendment"
            .into(),
    }]
}

pub fn record_charter_drift(
    plan: &Plan,
    charter: Option<&Charter>,
    store: &mut crate::state::Store,
) -> Result<Vec<Violation>, String> {
    let violations = charter_drift(plan, charter);
    for violation in &violations {
        store.record(
            "goal_drift",
            &serde_json::json!({"code":violation.code,"task":violation.task,"detail":violation.detail}),
            "Run stopped because its approved charter changed",
        )?;
    }
    Ok(violations)
}
