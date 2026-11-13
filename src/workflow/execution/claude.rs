//! Verify full-access initialization before forwarding a paid user prompt.
use crate::{
    process::{self, ControlReply, ProcessResult, ProcessSpec},
    protocol::claude_permissions::POLICY_ERROR,
};
use serde_json::{Value, json};
use std::path::PathBuf;

pub(super) async fn run(mut spec: ProcessSpec, stop: PathBuf) -> Result<ProcessResult, String> {
    let prompt = String::from_utf8(std::mem::take(&mut spec.input)).map_err(|e| e.to_string())?;
    spec.args
        .extend(["--input-format".into(), "stream-json".into()]);
    let initialize = json!({"type":"control_request","request_id":"permission-init","request":{"subtype":"initialize"}});
    spec.input = format!("{initialize}\n").into_bytes();
    let mut initialized = false;
    let handler = move |message: Value| {
        if message["type"] == "control_response"
            && message["response"]["request_id"] == "permission-init"
        {
            if initialized || message["response"]["subtype"] != "success" {
                return Err(format!(
                    "{POLICY_ERROR}full-access initialization failed or repeated; select native_callback."
                ));
            }
            let actual = message
                .pointer("/response/response/current_permission_mode")
                .and_then(Value::as_str);
            if actual != Some("bypassPermissions") {
                return Err(format!(
                    "{POLICY_ERROR}requested full_access, effective mode={}; no generation prompt was sent. Select native_callback in advanced CLI settings or review organization policy.",
                    actual.unwrap_or("unknown")
                ));
            }
            initialized = true;
            let user = json!({"type":"user","message":{"role":"user","content":prompt},"parent_tool_use_id":null,"session_id":""});
            return Ok(ControlReply {
                messages: vec![user],
                close_input: false,
            });
        }
        if message["type"] == "control_request" {
            return Err(format!(
                "{POLICY_ERROR}unexpected interactive permission request under full_access; use native_callback."
            ));
        }
        if message["type"] == "result" {
            return Ok(ControlReply {
                messages: vec![],
                close_input: true,
            });
        }
        Ok(ControlReply::default())
    };
    process::run_controlled(spec, Some(stop), Box::new(handler))
        .await
        .map_err(|error| match error.split_once(POLICY_ERROR) {
            Some((_, detail)) => format!("{POLICY_ERROR}{detail}"),
            None => error,
        })
}
