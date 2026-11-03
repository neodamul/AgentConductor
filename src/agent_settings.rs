//! GitHub #209: what differs between agent CLIs is data, not code. The shipped presets (`schemas/cli-io/v1/presets`,
//! embedded) and an operator's profile overrides have the shape of `acio.agent/1`; `adapters` builds argument lists
//! from them and no longer branches on the provider name.
use crate::{adapters::InputMode, protocol::Provider};
use serde::Deserialize;
use serde_json::Value;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
pub struct AgentSettings {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub capabilities: Value,
    pub invocation: Invocation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Invocation {
    #[serde(default)]
    pub executable: Option<String>,
    #[serde(default)]
    pub input: InputMode,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub full_access: Option<ArgsEdit>,
    #[serde(default)]
    pub session: SessionSettings,
    #[serde(default)]
    pub direct_folder: Option<DirectFolder>,
}

/// Append to the plain argv, or replace it as a whole.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ArgsEdit {
    Append { args: Vec<String> },
    ReplaceArgv { argv: Vec<String> },
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SessionSettings {
    #[serde(default)]
    pub model_args: Vec<String>,
    #[serde(default)]
    pub model_aliases: Vec<String>,
    #[serde(default)]
    pub selectors: Vec<String>,
    #[serde(default)]
    pub resume: Option<Resume>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum Resume {
    Append {
        args: Vec<String>,
    },
    ReplaceArgv {
        argv: Vec<String>,
        #[serde(default)]
        full_access_argv: Option<Vec<String>>,
        #[serde(default)]
        bare_argv: Option<Vec<String>>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirectFolder {
    pub after: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub skip_if_present: Vec<String>,
    #[serde(default)]
    pub missing_anchor_message: Option<String>,
}

fn preset_text(provider: Provider) -> Option<&'static str> {
    Some(match provider {
        Provider::Claude => include_str!("../schemas/cli-io/v1/presets/claude.json"),
        Provider::Codex => include_str!("../schemas/cli-io/v1/presets/codex.json"),
        Provider::OpenCode => include_str!("../schemas/cli-io/v1/presets/opencode.json"),
        Provider::Custom => return None,
    })
}

/// The shipped preset of a built-in provider; custom profiles have none.
pub fn preset(provider: Provider) -> Option<&'static AgentSettings> {
    static PRESETS: OnceLock<[Option<AgentSettings>; 3]> = OnceLock::new();
    let presets = PRESETS.get_or_init(|| {
        // The presets are validated against the schema by tests, so a failure here is a build defect.
        [Provider::Claude, Provider::Codex, Provider::OpenCode].map(|p| {
            preset_text(p).map(|text| serde_json::from_str(text).expect("embedded agent preset"))
        })
    });
    match provider {
        Provider::Claude => presets[0].as_ref(),
        Provider::Codex => presets[1].as_ref(),
        Provider::OpenCode => presets[2].as_ref(),
        Provider::Custom => None,
    }
}

/// JSON merge patch: objects merge key by key, `null` removes a key, anything else replaces.
pub fn merge(base: &mut Value, patch: &Value) {
    match (base, patch) {
        (Value::Object(base), Value::Object(patch)) => {
            for (key, value) in patch {
                if value.is_null() {
                    base.remove(key);
                } else {
                    merge(base.entry(key.clone()).or_insert(Value::Null), value);
                }
            }
        }
        (base, patch) => *base = patch.clone(),
    }
}

/// The settings of one profile: the preset of its provider, overridden by the profile's own `agent_settings`.
/// The result must be valid `acio.agent/1`.
pub fn settings_for(
    provider: Provider,
    overrides: Option<&Value>,
) -> Result<Option<AgentSettings>, String> {
    let base = preset(provider);
    let Some(overrides) = overrides else {
        return Ok(base.cloned());
    };
    // What defines the plain argv and full access (DD-131) is fixed by the preset: a profile may adjust how a
    // session is selected, not what "full access" means.
    if preset_text(provider).is_some() {
        for key in [
            "executable",
            "input",
            "args",
            "full_access",
            "direct_folder",
        ] {
            if overrides["invocation"].get(key).is_some() {
                return Err(format!(
                    "invocation.{key} is fixed by the preset and cannot be overridden"
                ));
            }
        }
    }
    let mut merged = match preset_text(provider) {
        Some(text) => serde_json::from_str(text).map_err(|e| e.to_string())?,
        None => serde_json::json!({"schema":"acio.agent/1"}),
    };
    merge(&mut merged, overrides);
    crate::cli_io::validate("agent", &merged)
        .map_err(|e| format!("invalid agent settings: {}", e.join("; ")))?;
    serde_json::from_value(merged)
        .map(Some)
        .map_err(|e| format!("invalid agent settings: {e}"))
}

pub fn full_access_args(settings: &AgentSettings) -> Option<Vec<String>> {
    Some(match settings.invocation.full_access.as_ref()? {
        ArgsEdit::Append { args } => settings
            .invocation
            .args
            .iter()
            .chain(args)
            .cloned()
            .collect(),
        ArgsEdit::ReplaceArgv { argv } => argv.clone(),
    })
}
