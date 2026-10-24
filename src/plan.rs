use serde::{Deserialize, Serialize};
use std::collections::HashSet;

mod profile;
mod task;
pub use profile::{
    AgentProfile, CatalogOrigin, CooperativeMode, ExecutionOption, Fallback, JsonCondition,
    ModelCatalog, ModelDiscovery, QuotaCommand,
};
pub use task::{Task, WorkspaceMode};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub goal: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub plan_rules: String,
    #[serde(default)]
    pub workspace_mode: WorkspaceMode,
    pub agents: Vec<AgentProfile>,
    pub tasks: Vec<Task>,
}

impl Plan {
    pub fn validate(&self) -> Result<(), String> {
        // DD-60: one to twelve tasks (R-53 supersedes the original fixed three).
        if self.goal.trim().is_empty() || !(1..=12).contains(&self.tasks.len()) {
            return Err("a nonempty goal and one to twelve tasks are required".into());
        }
        if self.plan_rules.len() > 64 * 1024 || self.plan_rules.contains('\0') {
            return Err("planning rules must be at most 64 KiB and contain no NUL".into());
        }
        let mut agents = HashSet::new();
        for agent in &self.agents {
            crate::quota::custom::validate_custom_rules(&agent.quota_rules)?;
            agent.validate_custom_configuration()?;
            if agent.parser != crate::protocol::Provider::Custom
                && (!agent.event_types.is_empty()
                    || agent.session_id_pointer.is_some()
                    || !agent.quota_rules.is_empty())
            {
                return Err("custom event mappings require a custom CLI profile".into());
            }
            if agent.quota_rules.iter().any(|rule| {
                agent
                    .event_types
                    .get(&rule.event_type)
                    .map_or(rule.event_type != "usage", |kind| {
                        *kind != crate::protocol::EventKind::Usage
                    })
            }) {
                return Err("custom quota events must be mapped to usage".into());
            }
            if let Some(quota) = &agent.quota_command
                && (agent.parser != crate::protocol::Provider::Custom
                    || quota.args.len() > 64
                    || !(1..=300).contains(&quota.timeout_seconds)
                    || quota
                        .args
                        .iter()
                        .any(|arg| arg.len() > 4096 || arg.contains('\0'))
                    || quota
                        .output_pointer
                        .as_ref()
                        .is_some_and(|pointer| !pointer.is_empty() && !pointer.starts_with('/')))
            {
                return Err("invalid custom quota command configuration".into());
            }
            if agent.event_types.len() > 64
                || agent.event_types.iter().any(|(name, _)| {
                    name.is_empty() || name.len() > 128 || name.chars().any(char::is_control)
                })
                || agent.session_id_pointer.as_ref().is_some_and(|pointer| {
                    pointer.len() > 256
                        || (!pointer.is_empty() && !pointer.starts_with('/'))
                        || pointer
                            .split('~')
                            .skip(1)
                            .any(|tail| !matches!(tail.chars().next(), Some('0' | '1')))
                })
            {
                return Err("invalid custom event mapping or session JSON Pointer".into());
            }
            if agent
                .environment
                .keys()
                .any(|key| key.starts_with("AC_CHECKPOINT_"))
            {
                return Err("checkpoint bootstrap environment is coordinator-owned".into());
            }
            if agent.cooperative == CooperativeMode::CheckpointV1
                && (agent.parser != crate::protocol::Provider::Custom
                    || agent
                        .args
                        .iter()
                        .map(|arg| arg.matches("{session_id}").count())
                        .sum::<usize>()
                        != 1)
            {
                return Err(
                    "checkpoint_v1 requires a custom CLI with one {session_id} argv binding".into(),
                );
            }
            let executable = crate::executable::program_name(&agent.executable);
            if agent.id.trim().is_empty()
                || !agents.insert(&agent.id)
                || executable.is_empty()
                || !(1..=3).contains(&agent.max_sessions)
                || !(1..=60).contains(&agent.quota_refresh_minutes)
                || matches!(
                    executable.as_str(),
                    "sh" | "bash" | "zsh" | "cmd" | "powershell" | "pwsh"
                )
            {
                return Err("invalid or duplicate agent profile".into());
            }
        }
        // Agents may name each other: execution stops at the first agent already tried,
        // so a chain where every account is limited waits for availability.
        for current in &self.agents {
            let Some(fallback) = &current.fallback else {
                continue;
            };
            let target = self
                .agents
                .iter()
                .find(|agent| agent.id == fallback.agent)
                .ok_or("unknown fallback agent")?;
            if current.quota_key() == target.quota_key() {
                return Err("fallback must use a different account quota group".into());
            }
            if let Some(model) = &fallback.model {
                target.validate_model(model)?;
            }
        }
        let mut ids = HashSet::new();
        let mut scopes: Vec<&str> = vec![];
        for task in &self.tasks {
            if task.id.trim().is_empty()
                || !ids.insert(&task.id)
                || !agents.contains(&task.agent)
                || task.prompt.trim().is_empty()
                || task.paths.is_empty()
                || task.test_command.is_empty()
                || task.test_command[0].is_empty()
            {
                return Err("invalid task identity, agent, prompt, scope or test command".into());
            }
            if let Some(model) = &task.model {
                self.agents
                    .iter()
                    .find(|a| a.id == task.agent)
                    .unwrap()
                    .validate_model(model)?;
            }
            let agent = self
                .agents
                .iter()
                .find(|agent| agent.id == task.agent)
                .unwrap();
            for (name, value) in &task.execution_options {
                let option = agent
                    .execution_options
                    .iter()
                    .find(|option| option.name == *name)
                    .ok_or_else(|| {
                        format!("task {} selects unknown option group {name}", task.id)
                    })?;
                let model = task
                    .execution_options
                    .get("model")
                    .map(String::as_str)
                    .or(task.model.as_deref())
                    .or_else(|| {
                        agent
                            .execution_options
                            .iter()
                            .find(|entry| entry.name == "model")
                            .and_then(|entry| entry.default.as_deref())
                    });
                let allowed = if name == "model" {
                    agent.validate_model(value).map(|_| true)
                } else {
                    option
                        .allowed_values(model)
                        .map(|allowed| allowed.contains(value))
                        .ok_or_else(|| {
                            format!("task {} uses {name} unavailable for its model", task.id)
                        })
                }?;
                if !allowed {
                    return Err(format!(
                        "task {} selects invalid {name} value for its model",
                        task.id
                    ));
                }
            }
            for option in &agent.execution_options {
                if option.name == "model" || matches!(option.name.as_str(), "resume" | "session") {
                    continue;
                }
                if task.execution_options.contains_key(&option.name) {
                    continue;
                }
                let Some(value) = option.default.as_deref() else {
                    continue;
                };
                let model = task
                    .execution_options
                    .get("model")
                    .map(String::as_str)
                    .or(task.model.as_deref())
                    .or_else(|| {
                        agent
                            .execution_options
                            .iter()
                            .find(|entry| entry.name == "model")
                            .and_then(|entry| entry.default.as_deref())
                    });
                if !option
                    .allowed_values(model)
                    .is_some_and(|allowed| allowed.iter().any(|allowed| allowed == value))
                {
                    return Err(format!(
                        "task {} has invalid default for {} on its model",
                        task.id, option.name
                    ));
                }
            }
            for path in &task.paths {
                if path.is_empty()
                    || path.contains(['\\', '*', '?', '[', ']'])
                    || path.split('/').any(|part| {
                        matches!(
                            part.to_ascii_lowercase().as_str(),
                            "" | "." | ".." | ".git" | ".agentconductor"
                        )
                    })
                    || scopes.iter().any(|other| overlaps(path, other))
                {
                    return Err(format!("unsafe or overlapping scope: {path}"));
                }
                scopes.push(path);
            }
        }
        let mut done = HashSet::new();
        while done.len() < self.tasks.len() {
            let previous = done.len();
            for task in &self.tasks {
                if let Some(id) = task.depends_on.iter().find(|id| !ids.contains(id)) {
                    return Err(format!("task {} has unknown dependency {id}", task.id));
                }
                if task.depends_on.iter().all(|id| done.contains(id)) {
                    done.insert(&task.id);
                }
            }
            if done.len() == previous {
                return Err("cyclic task dependencies".into());
            }
        }
        Ok(())
    }
}

/// Coordinators often write a directory scope as `docs/`. A slash-free directory path
/// already covers everything inside it (see `overlaps`), so trailing slashes are trimmed
/// before validation instead of rejecting the plan as an unsafe scope.
pub fn trim_directory_slashes(tasks: &mut [Task]) {
    for path in tasks.iter_mut().flat_map(|t| t.paths.iter_mut()) {
        let trimmed = path.trim_end_matches('/');
        if !trimmed.is_empty() && trimmed.len() != path.len() {
            *path = trimmed.to_owned();
        }
    }
}

/// Case-insensitive: the default macOS and Windows file systems name one file with both spellings (#79).
pub fn overlaps(left: &str, right: &str) -> bool {
    let (left, right) = (left.to_lowercase(), right.to_lowercase());
    left == right
        || left
            .strip_prefix(&right)
            .is_some_and(|tail| tail.starts_with('/'))
        || right
            .strip_prefix(&left)
            .is_some_and(|tail| tail.starts_with('/'))
}
