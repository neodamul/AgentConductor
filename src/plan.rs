use serde::{Deserialize, Serialize};
use std::collections::HashSet;

mod fallback_rule;
mod profile;
pub use fallback_rule::FallbackRule;
pub use profile::{
    AgentProfile, CatalogOrigin, CooperativeMode, DenialCondition, ExecutionOption, Fallback,
    InvalidJson, JsonCondition, ModelCatalog, ModelDiscovery, QuotaCommand,
};

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
    /// #992: only the verification command may fail; a worker must still succeed.
    #[serde(default, skip_serializing_if = "is_false")]
    pub test_expect_failure: bool,
    #[serde(default)]
    pub model: Option<String>,
    /// #409: absent inherits profile policy; false explicitly disables fallback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<FallbackRule>,
    /// DD-55: role from the operator's rules; omitted when absent so older digests hold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Operator-selected values for profile-declared argument groups.
    #[serde(
        default,
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        deserialize_with = "without_null_options"
    )]
    pub execution_options: std::collections::BTreeMap<String, String>,
    /// GitHub #210: the kind of work, set by the coordinator; omitted when absent so older digests hold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_type: Option<TaskType>,
}

impl Task {
    pub fn test_passed(
        &self,
        exit_code: Option<i32>,
        timed_out: bool,
        signal: Option<i32>,
    ) -> bool {
        !timed_out
            && signal.is_none()
            && exit_code.is_some_and(|code| {
                if self.test_expect_failure {
                    code > 0
                } else {
                    code == 0
                }
            })
    }
}

fn is_false(value: &bool) -> bool {
    !value
}

/// A strict output schema cannot omit an option, so a coordinator states "none" as null.
fn without_null_options<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<std::collections::BTreeMap<String, String>, D::Error> {
    let options = std::collections::BTreeMap::<String, Option<String>>::deserialize(deserializer)?;
    Ok(options
        .into_iter()
        .filter_map(|(k, v)| Some((k, v?)))
        .collect())
}

/// The kinds of work a task can be; model, effort and prompt conditions are chosen per kind (docs/cli-io-protocol.md §2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Extract,
    Summarize,
    Analyze,
    Code,
    Agentic,
    Creative,
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

mod validation;

/// A task, a verification task too, owns at least one path; the planner's scope correction retry keys on this text.
pub const EMPTY_PATHS: &str = "paths is empty; every task, a verification task too, must own at least one literal repo-relative file or directory";

/// GitHub #929: the planner's correction retry keys on this text.
pub const VACUOUS_CHECK: &str = "test_command passes even when the task produced nothing (for example `git status`, `true`, `echo`, `ls`); give a check that fails when the owned files are missing or wrong";

/// GitHub #929: a check that exits 0 whatever the task did (`git status`, `true`, `echo`,
/// `ls`, also wrapped in `sh -c`) verifies nothing, so a planned task may not use one.
pub fn is_vacuous_check(command: &[String]) -> bool {
    let name = |s: &str| {
        let base = s
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(s)
            .to_ascii_lowercase();
        base.strip_suffix(".exe").map(str::to_owned).unwrap_or(base)
    };
    let words: Vec<String> = match command {
        [shell, flag, script]
            if matches!(
                name(shell).as_str(),
                "sh" | "bash" | "zsh" | "cmd" | "powershell" | "pwsh"
            ) && matches!(flag.to_ascii_lowercase().as_str(), "-c" | "/c" | "-command") =>
        {
            script.split_whitespace().map(str::to_owned).collect()
        }
        _ => command.to_vec(),
    };
    match words.as_slice() {
        [] => false,
        [first, rest @ ..] => match name(first).as_str() {
            "true" | ":" | "echo" | "ls" | "dir" | "pwd" | "printf" => true,
            "git" => matches!(
                rest.first().map(String::as_str),
                Some("status" | "log" | "branch" | "show" | "rev-parse")
            ),
            _ => false,
        },
    }
}

/// GitHub #929: rejects planned tasks whose test_command cannot fail.
pub fn reject_vacuous_checks(tasks: &[Task]) -> Result<(), String> {
    match tasks.iter().find(|t| is_vacuous_check(&t.test_command)) {
        Some(task) => Err(format!("task {}: {VACUOUS_CHECK}", task.id)),
        None => Ok(()),
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
