use super::Task;
use std::collections::HashSet;

pub const EMPTY_PATHS: &str =
    "paths is empty; every task, including verification tasks, needs an owned repo-relative path";

pub(super) fn identity<'a>(
    task: &'a Task,
    agents: &HashSet<&String>,
    ids: &mut HashSet<&'a String>,
) -> Result<(), String> {
    let id = &task.id;
    if id.trim().is_empty() {
        return Err("task id is empty".into());
    }
    if !ids.insert(id) {
        return Err(format!("task {id}: duplicate task id"));
    }
    if !agents.contains(&task.agent) {
        return Err(format!("task {id}: unknown agent {}", task.agent));
    }
    if task.prompt.trim().is_empty() {
        return Err(format!("task {id}: prompt is empty"));
    }
    if task.paths.is_empty() {
        return Err(format!("task {id}: {EMPTY_PATHS}"));
    }
    if task
        .test_command
        .first()
        .is_none_or(|executable| executable.is_empty())
    {
        return Err(format!(
            "task {id}: test_command must begin with an executable"
        ));
    }
    Ok(())
}
