use serde::{Deserialize, Serialize};

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
    /// Optional custom CLI event names mapped to the coordinator's event contract.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub event_types: std::collections::BTreeMap<String, crate::protocol::EventKind>,
    /// JSON Pointer for a custom CLI's session ID; omitted profiles use `session_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id_pointer: Option<String>,
    /// Optional quota command for custom CLIs; stdout follows the normalized Snapshot schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota_command: Option<QuotaCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuotaCommand {
    pub args: Vec<String>,
    #[serde(default = "quota_timeout")]
    pub timeout_seconds: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_pointer: Option<String>,
}

fn quota_timeout() -> u16 {
    30
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
