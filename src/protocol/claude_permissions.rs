//! Effective Claude access is provider evidence, never an inference from argv.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
pub const POLICY_ERROR: &str = "Claude permission policy: ";
#[derive(Default)]
pub(super) struct Policy {
    pub full_access: bool,
    verified: bool,
    pub read_only_coordinator: bool,
    pending_reads: BTreeMap<String, String>,
    denied_reads: BTreeMap<String, String>,
    recovered_reads: BTreeSet<String>,
    incomplete_reads: bool,
}
impl Policy {
    fn read_tool(name: &str) -> bool {
        matches!(name, "Read" | "Grep" | "Glob")
    }

    fn observe_reads(&mut self, payload: &Value) {
        let Some(content) = payload
            .pointer("/message/content")
            .and_then(Value::as_array)
        else {
            return;
        };
        for item in content {
            if item["type"] == "tool_use" && item["name"].as_str().is_some_and(Self::read_tool) {
                let Some(id) = item["id"].as_str().filter(|id| id.len() <= 256) else {
                    self.incomplete_reads = true;
                    continue;
                };
                // Bound streamed recovery evidence; overflow cannot justify accepting a denied result.
                if self.pending_reads.len() >= 64 || self.pending_reads.contains_key(id) {
                    self.incomplete_reads = true;
                } else {
                    self.pending_reads
                        .insert(id.into(), item["name"].as_str().unwrap().into());
                }
            } else if item["type"] == "tool_result" {
                let Some(id) = item["tool_use_id"].as_str() else {
                    continue;
                };
                let Some(name) = self.pending_reads.remove(id) else {
                    continue;
                };
                if item
                    .get("is_error")
                    .is_some_and(|value| !value.is_boolean())
                {
                    self.incomplete_reads = true;
                    continue;
                }
                if item["is_error"] == true {
                    self.recovered_reads.remove(&name);
                    if self.denied_reads.len() >= 64 {
                        self.incomplete_reads = true;
                    } else {
                        self.denied_reads.insert(id.into(), name);
                    }
                } else {
                    self.recovered_reads.insert(name);
                }
            }
        }
    }

    fn reads_recovered(&self, payload: &Value) -> bool {
        if self.incomplete_reads || payload["is_error"] != false {
            return false;
        }
        payload["permission_denials"]
            .as_array()
            .is_some_and(|denials| {
                !denials.is_empty()
                    && denials.iter().all(|denial| {
                        let Some(name) = denial["tool_name"]
                            .as_str()
                            .filter(|name| Self::read_tool(name))
                        else {
                            return false;
                        };
                        let Some(id) = denial["tool_use_id"].as_str() else {
                            return false;
                        };
                        self.denied_reads.get(id).is_some_and(|tool| tool == name)
                            && self.recovered_reads.contains(name)
                    })
            })
    }

    pub fn normalize(&mut self, payload: Value) -> Value {
        if self.read_only_coordinator {
            self.observe_reads(&payload);
        }
        let init = payload["type"] == "system"
            && matches!(payload["subtype"].as_str(), Some("init" | "status"));
        let actual = payload["permissionMode"].as_str();
        let result = payload["type"] == "result";
        let denial = result
            && !(self.read_only_coordinator && self.reads_recovered(&payload))
            && payload
                .get("permission_denials")
                .is_some_and(|v| !v.is_null() && v.as_array().is_none_or(|a| !a.is_empty()));
        let message = if denial && self.read_only_coordinator {
            Some("Claude reported permission_denials during read-only coordination without verified read recovery; inspect the coordinator logs and snapshot source scope before retrying.".into())
        } else if denial {
            Some("Claude reported permission_denials; the task is not complete. Review the CLI policy or select native_callback in the agent's advanced settings.".to_string())
        } else if self.full_access && init && (payload["subtype"] == "init" || actual.is_some()) {
            self.verified = actual == Some("bypassPermissions");
            (!self.verified).then(||format!("requested full_access (bypassPermissions), effective permissionMode={}; check organization policy or select native_callback in the agent's advanced settings.",actual.unwrap_or("unknown")))
        } else if self.full_access
            && !self.verified
            && result
            && payload["is_error"] != true
            && payload
                .get("api_error")
                .is_none_or(|v| v.is_null() || v.as_str() == Some(""))
            && payload["api_error_status"].as_i64().is_none_or(|n| n < 400)
        {
            Some("full_access could not be verified from system/init; task completion is refused. Refresh the profile or select native_callback.".into())
        } else {
            None
        };
        match message {
            Some(message) => {
                json!({"type":"error","denied":true,"code":if denial{"permission_denials"}else{"permission_mode_mismatch"},"message":format!("{POLICY_ERROR}{message}"),"permissionMode":actual})
            }
            None => payload,
        }
    }
}
pub fn is_policy_error(error: &str) -> bool {
    error.starts_with(POLICY_ERROR)
}
pub fn profile_hash(profile: &crate::plan::AgentProfile) -> Result<String, String> {
    let mut profile = profile.clone();
    profile.catalog = None;
    profile.quota_refresh_minutes = 5;
    Ok(crate::workflow::digest(
        &serde_json::to_vec(&profile).map_err(|e| e.to_string())?,
    ))
}
pub fn observation(
    profile: &crate::plan::AgentProfile,
    reply: &Value,
    at: i64,
) -> Result<Value, String> {
    Ok(
        json!({"profile_hash":profile_hash(profile)?,"requested":profile.permission_mode,"effective":reply.pointer("/response/response/current_permission_mode").and_then(Value::as_str),"observed_at":at}),
    )
}
pub fn configuration_hint(
    profile: &crate::plan::AgentProfile,
    store: &crate::state::Store,
) -> Option<&'static str> {
    if profile.parser != super::Provider::Claude || profile.permission_mode != "full_access" {
        return None;
    }
    let observed = store
        .optional_document(&format!("claude_permission/{}", profile.id))
        .ok()
        .flatten();
    let matching = profile_hash(profile)
        .ok()
        .is_some_and(|hash| observed.as_ref().is_some_and(|v| v["profile_hash"] == hash));
    if !matching {
        return Some("startup.claude_permission_unverified");
    }
    match observed.as_ref().and_then(|v| v["effective"].as_str()) {
        Some("bypassPermissions") => None,
        Some(_) => Some("startup.claude_permission_mismatch"),
        None => Some("startup.claude_permission_unverified"),
    }
}

pub fn metadata_args(
    profile: &crate::plan::AgentProfile,
    args: Vec<&str>,
) -> Result<Vec<String>, String> {
    let mut args: Vec<String> = args.into_iter().map(String::from).collect();
    if profile.parser == super::Provider::Claude && profile.permission_mode == "full_access" {
        let base = crate::adapters::invocation(super::Provider::Claude, None, "unused")?.args;
        let full = crate::adapters::full_access_args(super::Provider::Claude)
            .ok_or("Claude full-access preset missing")?;
        args.extend(full.into_iter().skip(base.len()));
    }
    Ok(args)
}
pub fn write_metadata(
    profile: &crate::plan::AgentProfile,
    reply: &Value,
    dir: &std::path::Path,
    at: i64,
) -> Result<(), String> {
    if profile.parser == super::Provider::Claude
        && reply["type"] == "control_response"
        && reply["response"]["request_id"] == "catalog-init"
        && reply["response"]["subtype"] == "success"
    {
        let value = observation(profile, reply, at)?;
        std::fs::write(
            dir.join("permission.json"),
            serde_json::to_vec(&value).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
