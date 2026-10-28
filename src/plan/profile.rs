use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentProfile {
    pub id: String,
    pub executable: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub parser: crate::protocol::Provider,
    /// A wire protocol (acio) spoken by a custom CLI; it replaces the hand-written event mapping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<crate::protocol::Protocol>,
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
    #[serde(default, skip_serializing_if = "CatalogOrigin::is_discovered")]
    pub origin: CatalogOrigin,
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
            && !self
                .execution_options
                .iter()
                .any(|option| option.name == "model")
        {
            return Err("custom model selection requires a {model} argv placeholder".into());
        }
        Ok(())
    }

    pub fn validate_custom_configuration(&self) -> Result<(), String> {
        let custom = self.parser == crate::protocol::Provider::Custom;
        let pointer = |value: &str| value.is_empty() || value.starts_with('/');
        if self.protocol.is_some() {
            if !custom {
                return Err("the acio protocol requires a custom profile".into());
            }
            if !self.event_types.is_empty()
                || self.session_id_pointer.is_some()
                || self.result_pointer.is_some()
                || self.error_condition.is_some()
            {
                return Err("the acio protocol defines its own event mapping; remove the custom event mappings".into());
            }
        }
        if !custom
            && (self.model_discovery.is_some()
                || !self.execution_options.is_empty()
                || self.result_pointer.is_some()
                || self.error_condition.is_some())
        {
            return Err("custom CLI configuration requires a custom profile".into());
        }
        if let Some(discovery) = &self.model_discovery
            && (discovery.args.is_empty()
                || discovery.args.len() > 64
                || !(1..=300).contains(&discovery.timeout_seconds)
                || discovery
                    .args
                    .iter()
                    .any(|arg| arg.len() > 4096 || arg.contains('\0'))
                || !pointer(&discovery.models_pointer)
                || !pointer(&discovery.model_pointer))
        {
            return Err("invalid custom model discovery configuration".into());
        }
        let mut names = std::collections::HashSet::new();
        for option in &self.execution_options {
            if option.name.trim().is_empty()
                || !names.insert(&option.name)
                || option.args.is_empty()
                || option.args.len() > 32
                || option
                    .values
                    .iter()
                    .any(|value| value.is_empty() || value.starts_with('-'))
                || option.default.as_ref().is_some_and(|value| {
                    !option.values.is_empty() && !option.values.contains(value)
                })
                || option.name == "model" && !option.values.is_empty()
                || option.model_values.iter().any(|(model, values)| {
                    self.catalog
                        .as_ref()
                        .is_none_or(|catalog| !catalog.models.contains(model))
                        || values.iter().any(|value| !option.values.contains(value))
                })
            {
                return Err(format!("invalid execution option group {}", option.name));
            }
        }
        if self
            .execution_options
            .iter()
            .any(|option| option.name == "model")
            && self.args.iter().any(|arg| arg.contains("{model}"))
        {
            return Err(
                "custom model selection must use either its option group or an argv placeholder"
                    .into(),
            );
        }
        for value in self
            .result_pointer
            .iter()
            .chain(self.error_condition.iter().map(|c| &c.pointer))
        {
            if !pointer(value) {
                return Err("invalid custom result JSON Pointer".into());
            }
        }
        if self
            .error_condition
            .as_ref()
            .is_some_and(|condition| condition.equals.is_null())
        {
            return Err("custom error condition needs a non-null match value".into());
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
