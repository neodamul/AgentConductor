use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvalidJson {
    #[default]
    Fail,
    Text,
}

impl InvalidJson {
    pub fn is_fail(value: &Self) -> bool {
        *value == Self::Fail
    }
}
