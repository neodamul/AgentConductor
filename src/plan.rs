use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentProfile {
    pub id: String,
    pub executable: String,
    pub args: Vec<String>,
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
        if self.goal.trim().is_empty() || self.tasks.len() != 3 {
            return Err("a nonempty goal and exactly three tasks are required".into());
        }
        let mut agents = HashSet::new();
        for agent in &self.agents {
            let executable = agent.executable.rsplit('/').next().unwrap_or("");
            if agent.id.trim().is_empty()
                || !agents.insert(&agent.id)
                || executable.is_empty()
                || matches!(
                    executable,
                    "sh" | "bash" | "zsh" | "cmd" | "powershell" | "pwsh"
                )
            {
                return Err("invalid or duplicate agent profile".into());
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
