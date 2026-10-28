//! GitHub #207: the acio CLI I/O protocol (docs/cli-io-protocol.md). The schemas ship in the binary, a small
//! checker validates documents against them without a JSON Schema dependency, and `stream` checks the rules
//! over a whole captured stream.
mod check;
pub mod conformance;
mod pattern;
mod stream;
mod timestamp;

pub use stream::{Rules, validate_stream};

pub const KINDS: [&str; 5] = ["input", "event", "describe", "control", "agent"];

/// The JSON Schema text of one acio document kind.
pub fn schema(kind: &str) -> Result<&'static str, String> {
    Ok(match kind {
        "input" => include_str!("../schemas/cli-io/v1/input.schema.json"),
        "event" => include_str!("../schemas/cli-io/v1/event.schema.json"),
        "describe" => include_str!("../schemas/cli-io/v1/describe.schema.json"),
        "control" => include_str!("../schemas/cli-io/v1/control.schema.json"),
        "agent" => include_str!("../schemas/cli-io/v1/agent.schema.json"),
        _ => {
            return Err(format!(
                "unknown kind {kind}; use one of {}",
                KINDS.join(", ")
            ));
        }
    })
}

/// Checks one document; every violation is `<json pointer>: <reason>`.
pub fn validate(kind: &str, document: &serde_json::Value) -> Result<(), Vec<String>> {
    let root: serde_json::Value = serde_json::from_str(schema(kind).map_err(|e| vec![e])?)
        .map_err(|e| vec![format!("the embedded {kind} schema is invalid: {e}")])?;
    let errors = check::check(&root, document);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
