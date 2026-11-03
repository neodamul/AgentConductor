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
    /// Per-agent overrides of the shipped preset, in the shape of `acio.agent/1` (GitHub #209).
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
    /// GitHub #341: what to do with a stdout line that is not JSON (custom CLIs only).
    #[serde(default, skip_serializing_if = "InvalidJson::is_fail")]
    pub on_invalid_json: InvalidJson,
    /// GitHub #341: an event that means the CLI refused a tool (custom CLIs only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub denial_condition: Option<DenialCondition>,
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
    /// GitHub #325: the model this agent uses when it is the workspace coordinator; omitted means the CLI default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinator_model: Option<String>,
    /// GitHub #326: how the operator runs this CLI read-only (for the coordinator chat); the app cannot verify a custom CLI itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only_args: Option<Vec<String>>,
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

/// `fail` stops at the first line that is not JSON (and quotes it); `text` records such lines as progress text.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvalidJson {
    #[default]
    Fail,
    Text,
}

impl InvalidJson {
    fn is_fail(&self) -> bool {
        *self == Self::Fail
    }
}

/// GitHub #341: an event that says a tool was refused. `pointer` is checked against the event (optionally only
/// events of `event_type`) by `equals` or by `contains` (a case-insensitive message match); the tool and message
/// pointers name what to report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DenialCondition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    pub pointer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equals: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contains: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_pointer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_pointer: Option<String>,
}

impl DenialCondition {
    pub fn matches(&self, payload: &serde_json::Value) -> bool {
        if self
            .event_type
            .as_deref()
            .is_some_and(|kind| payload["type"].as_str() != Some(kind))
        {
            return false;
        }
        let Some(value) = payload.pointer(&self.pointer) else {
            return false;
        };
        match (&self.equals, &self.contains) {
            (Some(expected), _) => value == expected,
            (None, Some(text)) => value
                .as_str()
                .is_some_and(|v| v.to_lowercase().contains(&text.to_lowercase())),
            (None, None) => false,
        }
    }

    /// "permission denied: tool bash: <message>" from the pointers; a missing value is left out.
    pub fn describe(&self, payload: &serde_json::Value) -> String {
        let text = |pointer: &Option<String>| {
            pointer
                .as_deref()
                .and_then(|p| payload.pointer(p))
                .and_then(serde_json::Value::as_str)
                .map(String::from)
        };
        let mut out = String::from("permission denied");
        if let Some(tool) = text(&self.tool_pointer) {
            out += &format!(": tool {tool}");
        }
        if let Some(message) = text(&self.message_pointer) {
            out += &format!(": {message}");
        }
        out
    }
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
        if self.agent_settings.is_some() {
            crate::agent_settings::settings_for(self.parser, self.agent_settings.as_ref())?;
        }
        if self.protocol.is_some() {
            if !custom {
                return Err("the acio protocol requires a custom profile".into());
            }
            if !self.event_types.is_empty()
                || self.session_id_pointer.is_some()
                || self.result_pointer.is_some()
                || self.error_condition.is_some()
                || self.denial_condition.is_some()
                || self.on_invalid_json != InvalidJson::Fail
            {
                return Err("the acio protocol defines its own event mapping; remove the custom event mappings".into());
            }
        }
        if !custom
            && (self.model_discovery.is_some()
                || !self.execution_options.is_empty()
                || self.result_pointer.is_some()
                || self.error_condition.is_some()
                || self.denial_condition.is_some()
                || self.on_invalid_json != InvalidJson::Fail)
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
        if let Some(denial) = &self.denial_condition {
            let message_match =
                denial.equals.is_some() != denial.contains.as_ref().is_some_and(|c| !c.is_empty());
            if !message_match
                || denial
                    .equals
                    .as_ref()
                    .is_some_and(serde_json::Value::is_null)
                || !denial.pointer.starts_with('/')
                || !denial
                    .tool_pointer
                    .iter()
                    .chain(&denial.message_pointer)
                    .all(|p| pointer(p))
            {
                return Err("custom denial condition needs a JSON Pointer and exactly one of equals or contains".into());
            }
        }
        Ok(())
    }

    /// GitHub #341: a non-blocking hint when automatic approval is declared but nothing could show that a tool
    /// was refused, which is how a missing approval flag goes unnoticed.
    pub fn configuration_hint(&self) -> Option<&'static str> {
        (self.parser == crate::protocol::Provider::Custom
            && self.permission_mode == "operator_auto_approve"
            && self.protocol.is_none()
            && self.event_types.is_empty()
            && self.error_condition.is_none()
            && self.denial_condition.is_none())
        .then_some("startup.auto_approve_unconfigured")
    }

    pub fn quota_key(&self) -> &str {
        self.quota_group.as_deref().unwrap_or(&self.executable)
    }
}

fn unverified() -> String {
    "unverified".into()
}
