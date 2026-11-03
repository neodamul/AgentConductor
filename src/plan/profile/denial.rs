use serde::{Deserialize, Serialize};

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

    pub fn describe(&self, payload: &serde_json::Value) -> String {
        let text = |pointer: &Option<String>| {
            pointer
                .as_deref()
                .and_then(|p| payload.pointer(p))
                .and_then(serde_json::Value::as_str)
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
