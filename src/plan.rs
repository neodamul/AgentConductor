use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentProfile {
    pub id: String,
    pub executable: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub parser: crate::protocol::Provider,
    #[serde(default)]
    pub input_mode: crate::adapters::InputMode,
    #[serde(default)]
    pub environment: std::collections::BTreeMap<String, String>,
    #[serde(default = "unverified")]
    pub permission_mode: String,
    #[serde(default)]
    pub catalog: Option<ModelCatalog>,
    #[serde(default = "three")]
    pub max_sessions: usize,
    #[serde(default)]
    pub quota_group: Option<String>,
    #[serde(default)]
    pub fallback: Option<Fallback>,
    #[serde(default, skip_serializing_if = "CooperativeMode::is_disabled")]
    pub cooperative: CooperativeMode,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CooperativeMode {
    #[default]
    Disabled,
    CheckpointV1,
}

impl CooperativeMode {
    fn is_disabled(&self) -> bool {
        *self == Self::Disabled
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fallback {
    pub agent: String,
    #[serde(default)]
    pub model: Option<String>,
}

fn three() -> usize {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCatalog {
    pub models: Vec<String>,
    pub source: String,
    pub queried_at: i64,
    pub cli_version: String,
}

impl AgentProfile {
    pub fn validate_model(&self, model: &str) -> Result<(), String> {
        if model.is_empty()
            || model.starts_with('-')
            || model.chars().any(char::is_whitespace)
            || !self
                .catalog
                .as_ref()
                .is_some_and(|c| !c.source.is_empty() && c.models.iter().any(|m| m == model))
        {
            return Err("model must belong to an observed CLI catalog".into());
        }
        if self.parser == crate::protocol::Provider::Custom
            && !self.args.iter().any(|a| a.contains("{model}"))
        {
            return Err("custom model selection requires a {model} argv placeholder".into());
        }
        Ok(())
    }
    pub fn quota_key(&self) -> &str {
        self.quota_group.as_deref().unwrap_or(&self.executable)
    }
}

fn unverified() -> String {
    "unverified".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub id: String,
    pub agent: String,
    pub prompt: String,
    pub paths: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub test_command: Vec<String>,
    #[serde(default)]
    pub model: Option<String>,
    /// DD-55: role from the operator's rules; omitted when absent so older digests hold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub goal: String,
    #[serde(default)]
    pub workspace_mode: WorkspaceMode,
    pub agents: Vec<AgentProfile>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceMode {
    #[default]
    Shared,
    Worktree,
}

impl Plan {
    pub fn validate(&self) -> Result<(), String> {
        // DD-60: one to twelve tasks (R-53 supersedes the original fixed three).
        if self.goal.trim().is_empty() || !(1..=12).contains(&self.tasks.len()) {
            return Err("a nonempty goal and one to twelve tasks are required".into());
        }
        let mut agents = HashSet::new();
        for agent in &self.agents {
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
            let executable = agent.executable.rsplit('/').next().unwrap_or("");
            if agent.id.trim().is_empty()
                || !agents.insert(&agent.id)
                || executable.is_empty()
                || !(1..=3).contains(&agent.max_sessions)
                || matches!(
                    executable,
                    "sh" | "bash" | "zsh" | "cmd" | "powershell" | "pwsh"
                )
            {
                return Err("invalid or duplicate agent profile".into());
            }
        }
        for origin in &self.agents {
            let mut visited = HashSet::from([origin.id.as_str()]);
            let mut current = origin;
            while let Some(fallback) = &current.fallback {
                let target = self
                    .agents
                    .iter()
                    .find(|agent| agent.id == fallback.agent)
                    .ok_or("unknown fallback agent")?;
                if !visited.insert(target.id.as_str()) {
                    return Err("cyclic fallback policy".into());
                }
                if current.quota_key() == target.quota_key() {
                    return Err("fallback must use a different account quota group".into());
                }
                if let Some(model) = &fallback.model {
                    target.validate_model(model)?;
                }
                current = target;
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
            for path in &task.paths {
                if path.is_empty()
                    || path.contains(['\\', '*', '?', '[', ']'])
                    || path
                        .split('/')
                        .any(|part| matches!(part, "" | "." | ".." | ".git" | ".agentconductor"))
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
                if task.depends_on.iter().any(|id| !ids.contains(id)) {
                    return Err("unknown dependency".into());
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

pub fn overlaps(left: &str, right: &str) -> bool {
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|tail| tail.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|tail| tail.starts_with('/'))
}
