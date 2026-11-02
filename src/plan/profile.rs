use serde::{Deserialize, Serialize};

mod validation;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentProfile {
    pub id: String,
    pub executable: String,
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinator_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only_args: Option<Vec<String>>,
    #[serde(default)]
    pub parser: crate::protocol::Provider,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<crate::protocol::Protocol>,
    /// Optional agent capability metadata used to resolve task-specific model and effort defaults.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_settings: Option<serde_json::Value>,
    #[serde(default)]
    pub input_mode: crate::adapters::InputMode,
    #[serde(default)]
    pub environment: std::collections::BTreeMap<String, String>,
    #[serde(default = "unverified")]
    pub permission_mode: String,
    #[serde(default)]
    pub catalog: Option<ModelCatalog>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_discovery: Option<ModelDiscovery>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub execution_options: Vec<ExecutionOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_pointer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<JsonCondition>,
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
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub event_types: std::collections::BTreeMap<String, crate::protocol::EventKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id_pointer: Option<String>,
    /// Optional CLI argv for on-demand quota refresh; stdout is normalized Snapshot JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota_command: Option<QuotaCommand>,
    /// Declarative mappings for quota metadata emitted in owned JSONL sessions.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_quota_rules"
    )]
    pub quota_rules: Vec<crate::quota::custom::CustomQuotaRule>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelDiscovery {
    pub args: Vec<String>,
    pub models_pointer: String,
    #[serde(default)]
    pub model_pointer: String,
    #[serde(default = "quota_timeout")]
    pub timeout_seconds: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionOption {
    pub name: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub values: Vec<String>,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub model_values: std::collections::BTreeMap<String, Vec<String>>,
}

impl ExecutionOption {
    pub fn allowed_values(&self, model: Option<&str>) -> Option<&Vec<String>> {
        if self.model_values.is_empty() {
            Some(&self.values)
        } else {
            self.model_values.get(model?)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonCondition {
    pub pointer: String,
    pub equals: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogOrigin {
    Manual,
    #[default]
    Discovered,
}

impl CatalogOrigin {
    fn is_discovered(&self) -> bool {
        *self == Self::Discovered
    }
}

fn quota_timeout() -> u16 {
    30
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
    #[serde(default, skip_serializing_if = "CatalogOrigin::is_discovered")]
    pub origin: CatalogOrigin,
}

fn unverified() -> String {
    "unverified".into()
}
