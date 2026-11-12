//! DD-143: action-specific coordinator responses keep inspection non-executing.
use serde::{Deserialize, Deserializer};
use serde_json::Value;
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Selection {
    Read {
        server: String,
        tool: String,
        arguments: Value,
    },
    Helper {
        server: String,
        tool: String,
        arguments: Value,
    },
    Inspect {
        server: String,
        #[serde(default)]
        tool: Option<String>,
        #[serde(default, rename = "arguments", deserialize_with = "empty_arguments")]
        _arguments: (),
    },
    Unavailable {
        reason: String,
    },
}
fn empty_arguments<'de, D: Deserializer<'de>>(deserializer: D) -> Result<(), D::Error> {
    let value = Value::deserialize(deserializer)?;
    if value.is_null() || value.as_object().is_some_and(|object| object.is_empty()) {
        Ok(())
    } else {
        Err(serde::de::Error::custom(
            "inspect arguments must be null or an empty object",
        ))
    }
}
#[cfg(test)]
#[path = "../../tests/support/mcp_selection.rs"]
mod tests;
