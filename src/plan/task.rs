use super::Fallback;
use serde::{Deserialize, Serialize};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Fallback>,
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
    /// GitHub #210: work category used to resolve task-specific model settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_type: Option<TaskType>,
}

/// Strict structured output represents an unset effort as null; omit it from the runtime map.
fn without_null_options<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<std::collections::BTreeMap<String, String>, D::Error> {
    let options = std::collections::BTreeMap::<String, Option<String>>::deserialize(deserializer)?;
    Ok(options
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| (key, value)))
        .collect())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Extract,
    Summarize,
    Analyze,
    Code,
    Agentic,
    Creative,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceMode {
    #[default]
    Shared,
    Worktree,
}
