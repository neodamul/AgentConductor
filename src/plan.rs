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
