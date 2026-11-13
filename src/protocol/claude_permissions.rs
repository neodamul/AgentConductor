//! Effective Claude access is provider evidence, never an inference from argv.
use serde_json::{Value, json};
pub const POLICY_ERROR: &str = "Claude permission policy: ";
#[derive(Default)]
pub(super) struct Policy {
    pub full_access: bool,
    verified: bool,
}
impl Policy {
    pub fn normalize(&mut self, payload: Value) -> Value {
        let init = payload["type"] == "system"
            && matches!(payload["subtype"].as_str(), Some("init" | "status"));
        let actual = payload["permissionMode"].as_str();
        let result = payload["type"] == "result";
        let denial = result
            && payload
                .get("permission_denials")
                .is_some_and(|v| !v.is_null() && v.as_array().is_none_or(|a| !a.is_empty()));
        let message = if denial {
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
