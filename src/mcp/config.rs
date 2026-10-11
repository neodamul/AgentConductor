use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
fn enabled() -> bool {
    true
}
fn timeout() -> u64 {
    5000
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum Server {
    Local {
        command: Vec<String>,
        #[serde(default)]
        cwd: Option<String>,
        #[serde(default)]
        environment: BTreeMap<String, String>,
        #[serde(default = "enabled")]
        enabled: bool,
        #[serde(default = "timeout")]
        timeout: u64,
    },
    Remote {
        url: String,
        #[serde(default)]
        headers: BTreeMap<String, String>,
        #[serde(default)]
        oauth: Option<Value>,
        #[serde(default = "enabled")]
        enabled: bool,
        #[serde(default = "timeout")]
        timeout: u64,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub server: String,
    pub tool: String,
    pub arguments: Value,
    pub read_only: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub mcp: BTreeMap<String, Server>,
    #[serde(default)]
    pub goal_sources: BTreeMap<String, Binding>,
}
impl Settings {
    pub fn parse(text: &str) -> Result<Self, String> {
        let v = super::jsonc::parse(text)?;
        if !v.is_object() {
            return Err("MCP settings must be an object".into());
        }
        let settings: Self = serde_json::from_value(v)
            .map_err(|_| "invalid OpenCode MCP configuration".to_string())?;
        settings.validate()?;
        Ok(settings)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.mcp.len() > 8 {
            return Err("at most eight MCP servers".into());
        }
        for (name, s) in &self.mcp {
            if name.is_empty()
                || name.len() > 64
                || !name
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
            {
                return Err("invalid MCP server name".into());
            }
            if !(100..=60000).contains(&s.timeout()) {
                return Err("MCP timeout must be 100–60000 ms".into());
            }
            match s {
                Server::Local {
                    command,
                    environment,
                    ..
                } => {
                    if command.is_empty()
                        || command[0].is_empty()
                        || command.len() > 32
                        || command.iter().any(|a| a.contains(['\n', '\r', '\0']))
                    {
                        return Err("invalid MCP command array".into());
                    }
                    check_values(environment)?;
                }
                Server::Remote {
                    url,
                    headers,
                    oauth,
                    ..
                } => {
                    let safe = url.starts_with("https://")
                        || url.starts_with("http://127.0.0.1:")
                        || url.starts_with("http://localhost:");
                    if !safe || url.contains(['@', '#', '\r', '\n', '\0', '?']) {
                        return Err("MCP URL must use HTTPS (HTTP only for loopback), without credentials/query/fragment".into());
                    }
                    check_values(headers)?;
                    if let Some(v) = oauth
                        && !v.is_boolean()
                        && !v.is_object()
                    {
                        return Err("invalid OpenCode oauth configuration".into());
                    }
                    if let Some(v) = oauth {
                        for key in ["clientSecret", "clientId"] {
                            if let Some(secret) = v.get(key).and_then(Value::as_str)
                                && !is_env(secret)
                            {
                                return Err("OAuth credentials must use {env:VARIABLE} references; authenticate through a local proxy".into());
                            }
                        }
                    }
                }
            }
        }
        for (kind, b) in &self.goal_sources {
            if !["jira", "confluence"].contains(&kind.as_str())
                || !self.mcp.contains_key(&b.server)
                || !b.read_only
                || b.tool.is_empty()
                || b.tool.len() > 128
                || !b.arguments.is_object()
                || !b.arguments.to_string().contains("{id}")
            {
                return Err("goal sources require jira/confluence, an existing server, an exact read-only tool and {id} arguments".into());
            }
        }
        Ok(())
    }
    pub fn digest(&self) -> Result<String, String> {
        Ok(crate::workflow::digest(
            &serde_json::to_vec(self).map_err(|e| e.to_string())?,
        ))
    }
}
fn check_values(values: &BTreeMap<String, String>) -> Result<(), String> {
    for (k, v) in values {
        if k.is_empty()
            || !k
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
            || !is_env(v)
        {
            return Err(
                "MCP credentials/environment values must use {env:VARIABLE} references".into(),
            );
        }
    }
    Ok(())
}
fn is_env(s: &str) -> bool {
    s.strip_prefix("{env:")
        .and_then(|s| s.strip_suffix('}'))
        .is_some_and(|s| !s.is_empty() && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_'))
}
pub(super) fn expand(s: &str) -> Result<String, String> {
    if let Some(inner) = s.strip_prefix("Bearer ") {
        return expand(inner).map(|v| format!("Bearer {v}"));
    }
    if let Some(inner) = s.strip_prefix("Basic ") {
        return expand(inner).map(|v| format!("Basic {v}"));
    }
    if !is_env(s) {
        return Err("invalid credential reference".into());
    }
    let key = &s[5..s.len() - 1];
    let v = std::env::var(key).map_err(|_| format!("MCP environment variable {key} is not set"))?;
    if v.contains(['\r', '\n', '\0']) {
        return Err("invalid credential value".into());
    }
    Ok(v)
}
impl Server {
    pub(super) fn enabled(&self) -> bool {
        match self {
            Self::Local { enabled, .. } | Self::Remote { enabled, .. } => *enabled,
        }
    }
    pub(super) fn timeout(&self) -> u64 {
        match self {
            Self::Local { timeout, .. } | Self::Remote { timeout, .. } => *timeout,
        }
    }
}
