use serde::{Deserialize, Serialize};
use std::collections::HashSet;

mod profile;
mod task;
pub use task::{Task, WorkspaceMode};

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
    #[serde(default = "five_minutes", skip_serializing_if = "default_cadence")]
    pub quota_refresh_minutes: u16,
    #[serde(default)]
    pub quota_group: Option<String>,
    #[serde(default)]
    pub fallback: Option<Fallback>,
    #[serde(default, skip_serializing_if = "CooperativeMode::is_disabled")]
    pub cooperative: CooperativeMode,
    /// Custom CLI event names mapped to the normalized event contract.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub event_types: std::collections::BTreeMap<String, crate::protocol::EventKind>,
    /// Declarative JSONL usage mappings; these read process output only.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_quota_rules"
    )]
    pub quota_rules: Vec<crate::quota::custom::CustomQuotaRule>,
    /// JSON Pointer for custom CLI session IDs; `None` uses `session_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id_pointer: Option<String>,
}

fn deserialize_quota_rules<'de, D>(
    deserializer: D,
) -> Result<Vec<crate::quota::custom::CustomQuotaRule>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let rules = Vec::<crate::quota::custom::CustomQuotaRule>::deserialize(deserializer)?;
    crate::quota::custom::validate_custom_rules(&rules).map_err(serde::de::Error::custom)?;
    Ok(rules)
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

fn five_minutes() -> u16 {
    5
}
fn default_cadence(value: &u16) -> bool {
    *value == 5
}

fn three() -> usize {
    3
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCatalog {
    pub models: Vec<String>,
    pub source: String,
    pub queried_at: i64,
    pub cli_version: String,
}

fn unverified() -> String {
    "unverified".into()
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

impl Plan {
    pub fn validate(&self) -> Result<(), String> {
        // DD-60: one to twelve tasks (R-53 supersedes the original fixed three).
        if self.goal.trim().is_empty() || !(1..=12).contains(&self.tasks.len()) {
            return Err("a nonempty goal and one to twelve tasks are required".into());
        }
        let mut agents = HashSet::new();
        for agent in &self.agents {
            crate::quota::custom::validate_custom_rules(&agent.quota_rules)?;
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

pub fn overlaps(left: &str, right: &str) -> bool {
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|tail| tail.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|tail| tail.starts_with('/'))
}
