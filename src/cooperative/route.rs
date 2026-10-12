//! DD-100: the operator chooses Local Rust or agentmemory preferred for coordination signals.
//! The directive is already durable in the Rust ledger before any route is attempted, so a
//! failed memory adapter falls back to local polling without changing the directive identity.
use super::Directive;
use crate::{memory, state::Store};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const FILE: &str = "signal-transport.json";
const LOCK: &str = "signal-memory.lock";

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    #[default]
    Local,
    Agentmemory,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Preference {
    pub transport: Transport,
    pub port: u16,
}
impl Default for Preference {
    fn default() -> Self {
        Self {
            transport: Transport::Local,
            port: 3111,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Route {
    pub directive: String,
    pub preference: Transport,
    pub effective: Transport,
    pub memory_id: Option<String>,
    pub cause: Option<String>,
}
#[derive(Default, Serialize, Deserialize)]
struct Settings {
    #[serde(default)]
    preference: Preference,
    #[serde(default)]
    last: Option<Route>,
}

fn read(workspace: &Path) -> Settings {
    fs::read(workspace.join(FILE))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}
fn write(workspace: &Path, settings: &Settings) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(workspace.join(FILE), bytes).map_err(|e| e.to_string())
}
/// Missing or unreadable settings mean Local Rust (DD-100 default).
pub fn load_preference(workspace: &Path) -> Preference {
    read(workspace).preference
}
pub fn save_preference(workspace: &Path, preference: &Preference) -> Result<(), String> {
    if preference.port == 0 {
        return Err("agentmemory port must be nonzero".into());
    }
    let mut settings = read(workspace);
    settings.preference = preference.clone();
    write(workspace, &settings)
}
/// The most recent route outcome for Settings.
pub fn last(workspace: &Path) -> Option<Route> {
    read(workspace).last
}

/// Routes a directive that `issue` already persisted. Idempotent per directive: a recorded
/// route is returned unchanged, so a retry never publishes the instruction twice.
pub fn deliver(
    store: &mut Store,
    directive: &Directive,
    preference: &Preference,
    workspace: &Path,
) -> Result<Route, String> {
    let key = format!("signal_route/{}", directive.id);
    if let Some(prior) = store.optional_document(&key)? {
        return serde_json::from_value(prior).map_err(|e| e.to_string());
    }
    let mut route = Route {
        directive: directive.id.clone(),
        preference: preference.transport,
        effective: Transport::Local,
        memory_id: None,
        cause: None,
    };
    if preference.transport == Transport::Agentmemory {
        match publish(directive, preference.port, workspace) {
            Ok(id) => {
                route.effective = Transport::Agentmemory;
                route.memory_id = Some(id);
            }
            // The recipient keeps polling the local ledger with the same directive ID.
            Err(e) => route.cause = Some(e.chars().take(300).collect()),
        }
    }
    let value = serde_json::to_value(&route).map_err(|e| e.to_string())?;
    store.record_once(
        &key,
        &value,
        "Recorded the signal route; execution and approval permissions unchanged",
    )?;
    let mut settings = read(workspace);
    settings.last = Some(route.clone());
    // Settings display only; the store above is authoritative.
    let _ = write(workspace, &settings);
    Ok(route)
}

/// At most one external memory operation per workspace; the client itself is bounded to 8 s.
fn publish(directive: &Directive, port: u16, workspace: &Path) -> Result<String, String> {
    let lock = workspace.join(LOCK);
    let acquire = || {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock)
    };
    if acquire().is_err() {
        // A live operation ends within 8 s; an older lock was left by a crashed process.
        let stale = fs::metadata(&lock)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age.as_secs() > 30);
        if !stale || fs::remove_file(&lock).is_err() || acquire().is_err() {
            return Err("another agentmemory signal operation is in flight".into());
        }
    }
    let result = memory::Client::from_environment(port).and_then(|client| {
        client.publish(&memory::checkpoints::Envelope::Directive {
            directive: directive.clone(),
        })
    });
    let _ = fs::remove_file(&lock);
    result
        .map(|receipt| receipt.memory_id)
        .map_err(|e| format!("agentmemory unavailable: {e}"))
}
