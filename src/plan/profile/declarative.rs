use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    pub model_values: BTreeMap<String, Vec<String>>,
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
    pub(super) fn is_discovered(&self) -> bool {
        *self == Self::Discovered
    }
}

fn quota_timeout() -> u16 {
    30
}
