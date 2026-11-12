//! GitHub #904: the last successful read call per reference kind, reused before asking the coordinator.
//! Workspace-local (mcp/read-calls.json); arguments only, never headers or environment values.
use super::Reference;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};
const MARK: &str = "\u{0}reference";
const MARK_NUMBER: &str = "\u{0}reference-number";
const FILE: &str = "read-calls.json";
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ReadCall {
    pub server: String,
    pub tool: String,
    /// SHA-256 of the server's connection settings: an edited server never reuses an old call.
    pub server_sha256: String,
    /// SHA-256 of the tool's input schema when the call succeeded.
    pub schema_sha256: String,
    /// Arguments with the reference ID replaced by a marker.
    pub template: Value,
}
fn template(value: &Value, id: &str) -> (Value, bool) {
    match value {
        Value::String(s) if s == id => (Value::String(MARK.into()), true),
        Value::Number(n) if n.to_string() == id => (Value::String(MARK_NUMBER.into()), true),
        Value::Array(a) => {
            let mut found = false;
            let items = a
                .iter()
                .map(|v| {
                    let (v, f) = template(v, id);
                    found |= f;
                    v
                })
                .collect();
            (Value::Array(items), found)
        }
        Value::Object(o) => {
            let mut found = false;
            let items = o
                .iter()
                .map(|(k, v)| {
                    let (v, f) = template(v, id);
                    found |= f;
                    (k.clone(), v)
                })
                .collect();
            (Value::Object(items), found)
        }
        other => (other.clone(), false),
    }
}
/// The stored arguments for `id`; numeric IDs return to numbers where the original call used one.
pub(super) fn arguments(template: &Value, id: &str) -> Value {
    match template {
        Value::String(s) if s == MARK => Value::String(id.into()),
        Value::String(s) if s == MARK_NUMBER => id
            .parse::<u64>()
            .map(Value::from)
            .unwrap_or_else(|_| Value::String(id.into())),
        Value::Array(a) => Value::Array(a.iter().map(|v| arguments(v, id)).collect()),
        Value::Object(o) => Value::Object(
            o.iter()
                .map(|(k, v)| (k.clone(), arguments(v, id)))
                .collect(),
        ),
        other => other.clone(),
    }
}
pub(super) fn digest(value: &Value) -> String {
    crate::workflow::digest(value.to_string().as_bytes())
}
fn load_all(root: &Path) -> BTreeMap<String, ReadCall> {
    super::load(&root.join("mcp").join(FILE))
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
        .unwrap_or_default()
}
fn save_all(root: &Path, calls: &BTreeMap<String, ReadCall>) {
    if let Ok(data) = serde_json::to_vec_pretty(calls) {
        super::progress::write_atomic(&root.join("mcp").join(FILE), &data);
    }
}
pub(super) fn get(root: &Path, kind: &str) -> Option<ReadCall> {
    load_all(root).remove(kind)
}
/// Stores a successful read call; a call whose arguments do not carry the reference ID is not reusable.
pub(super) fn remember(
    root: &Path,
    r: &Reference,
    server: &str,
    server_sha256: String,
    tool: &str,
    schema: &Value,
    args: &Value,
) {
    let (template, found) = template(args, &r.id);
    if !found {
        return;
    }
    let call = ReadCall {
        server: server.into(),
        tool: tool.into(),
        server_sha256,
        schema_sha256: digest(schema),
        template,
    };
    let mut calls = load_all(root);
    if calls.get(&r.kind) != Some(&call) {
        calls.insert(r.kind.clone(), call);
        save_all(root, &calls);
    }
}
pub(super) fn forget(root: &Path, kind: &str) {
    let mut calls = load_all(root);
    if calls.remove(kind).is_some() {
        save_all(root, &calls);
    }
}
#[cfg(test)]
#[path = "../../tests/unit/mcp_read_calls.rs"]
mod tests;
