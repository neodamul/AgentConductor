use super::CooperativeMode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    pub environment: BTreeMap<String, String>,
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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub event_types: BTreeMap<String, crate::protocol::EventKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id_pointer: Option<String>,
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
pub struct Fallback {
    pub agent: String,
    #[serde(default)]
    pub model: Option<String>,
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
    pub fn validate_custom_config(&self) -> Result<(), String> {
        if self.parser != crate::protocol::Provider::Custom
            && (!self.event_types.is_empty()
                || self.session_id_pointer.is_some()
                || self.quota_command.is_some())
        {
            return Err("custom event and quota settings require parser=custom".into());
        }
        if self.event_types.len() > 64
            || self.event_types.iter().any(|(name, kind)| {
                name.trim().is_empty()
                    || name.len() > 256
                    || *kind == crate::protocol::EventKind::Unknown
            })
            || self
                .session_id_pointer
                .as_deref()
                .is_some_and(|p| !valid_pointer(p))
        {
            return Err("invalid custom event mapping or session JSON Pointer".into());
        }
        if let Some(command) = &self.quota_command
            && (command.args.len() > 64
                || !(1..=300).contains(&command.timeout_seconds)
                || command
                    .args
                    .iter()
                    .any(|arg| arg.len() > 4096 || arg.contains('\0'))
                || command
                    .output_pointer
                    .as_deref()
                    .is_some_and(|p| !valid_pointer(p)))
        {
            return Err("invalid custom quota command configuration".into());
        }
        Ok(())
    }

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

fn valid_pointer(pointer: &str) -> bool {
    pointer.is_empty()
        || (pointer.starts_with('/')
            && pointer.len() <= 512
            && pointer.split('/').skip(1).all(|part| {
                let mut chars = part.chars();
                while let Some(ch) = chars.next() {
                    if ch == '~' && !matches!(chars.next(), Some('0' | '1')) {
                        return false;
                    }
                }
                true
            }))
}

fn quota_timeout() -> u16 {
    30
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
fn unverified() -> String {
    "unverified".into()
}
