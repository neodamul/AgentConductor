use crate::process::{ControlHandler, ControlReply};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct ToolDecision {
    pub allow: bool,
    pub reason: String,
}

impl ToolDecision {
    pub fn deny(reason: &str) -> Self {
        Self {
            allow: false,
            reason: reason.into(),
        }
    }
}

pub type Evaluator = Box<dyn FnMut(&str, &Value) -> Result<ToolDecision, String> + Send>;

pub struct Controller {
    prompt: String,
    initialized: bool,
    result_seen: bool,
    state: Option<String>,
    requests: HashSet<String>,
    permissions: HashMap<String, (String, ToolDecision)>,
    evaluator: Evaluator,
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value[key]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 256)
        .ok_or_else(|| format!("missing or invalid control {key}"))
}

impl Controller {
    pub fn new(prompt: &str, evaluator: Evaluator) -> Result<Self, String> {
        if prompt.trim().is_empty() || prompt.len() > 128 * 1024 {
            return Err("controlled prompt must contain 1..131072 bytes".into());
        }
        Ok(Self {
            prompt: prompt.into(),
            initialized: false,
            result_seen: false,
            state: None,
            requests: HashSet::new(),
            permissions: HashMap::new(),
            evaluator,
        })
    }

    pub fn initial_input(&self) -> Result<Vec<u8>, String> {
        let mut bytes=serde_json::to_vec(&json!({"type":"control_request","request_id":"ac-initialize","request":{"subtype":"initialize","hooks":{"PreToolUse":[{"matcher":null,"hookCallbackIds":["ac-pre-tool"],"timeout":5}]},"skills":[]}})).map_err(|e|e.to_string())?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    pub fn handler(mut self) -> ControlHandler {
        Box::new(move |message| self.handle(message))
    }

    pub fn handle(&mut self, message: Value) -> Result<ControlReply, String> {
        if !message.is_object() {
            return Err("native control message must be an object".into());
        }
        match message["type"].as_str() {
            Some("control_response") if message["response"]["request_id"] == "ac-initialize" => {
                if self.initialized || message["response"]["subtype"] != "success" {
                    return Err("native control initialization failed or repeated".into());
                }
                self.initialized = true;
                Ok(ControlReply {
                    messages: vec![
                        json!({"type":"user","message":{"role":"user","content":self.prompt},"parent_tool_use_id":null,"session_id":""}),
                    ],
                    close_input: false,
                })
            }
            Some("control_request") => {
                if !self.initialized {
                    return Err("native permission request preceded initialization".into());
                }
                let id = text(&message, "request_id")?;
                if self.requests.len() >= 2048 || !self.requests.insert(id.into()) {
                    return Err("duplicate or excessive native control requests".into());
                }
                let request = &message["request"];
                let response = match request["subtype"].as_str() {
                    Some("hook_callback") => {
                        if request["callback_id"] != "ac-pre-tool" {
                            return Err("unknown native hook callback".into());
                        }
                        let tool = text(&request["input"], "tool_name")?;
                        let input = &request["input"]["tool_input"];
                        if !input.is_object() {
                            return Err("native tool input must be an object".into());
                        }
                        let decision = (self.evaluator)(tool, input)?;
                        if decision.reason.trim().is_empty() || decision.reason.len() > 8192 {
                            return Err("invalid permission decision reason".into());
                        }
                        let use_id = request["tool_use_id"]
                            .as_str()
                            .or_else(|| request["input"]["tool_use_id"].as_str())
                            .filter(|s| !s.is_empty() && s.len() <= 256)
                            .ok_or("native tool-use identity missing")?;
                        if self.permissions.len() >= 256 || self.permissions.contains_key(use_id) {
                            return Err(
                                "ambiguous or excessive outstanding native permission identities"
                                    .into(),
                            );
                        }
                        let fingerprint = crate::workflow::digest(
                            &serde_json::to_vec(&(tool, input)).map_err(|e| e.to_string())?,
                        );
                        self.permissions
                            .insert(use_id.into(), (fingerprint, decision.clone()));
                        json!({"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":if decision.allow {"allow"}else{"deny"},"permissionDecisionReason":decision.reason}})
                    }
                    Some("can_use_tool") => {
                        let tool = text(request, "tool_name")?;
                        let input = &request["input"];
                        let use_id = text(request, "tool_use_id")?;
                        let fingerprint = crate::workflow::digest(
                            &serde_json::to_vec(&(tool, input)).map_err(|e| e.to_string())?,
                        );
                        let decision = self
                            .permissions
                            .remove(use_id)
                            .filter(|(expected, _)| *expected == fingerprint)
                            .map(|(_, decision)| decision)
                            .unwrap_or_else(|| {
                                ToolDecision::deny("No matching pre-execution permission decision")
                            });
                        if decision.allow {
                            json!({"behavior":"allow","updatedInput":input})
                        } else {
                            json!({"behavior":"deny","message":decision.reason,"interrupt":true})
                        }
                    }
                    _ => return Err("unsupported native control request; execution refused".into()),
                };
                Ok(ControlReply {
                    messages: vec![
                        json!({"type":"control_response","response":{"subtype":"success","request_id":id,"response":response}}),
                    ],
                    close_input: false,
                })
            }
            Some("system") if message["subtype"] == "session_state_changed" => {
                let state = text(&message, "state")?;
                if !matches!(state, "running" | "idle" | "requires_action") {
                    return Err("unknown native session state".into());
                }
                self.state = Some(state.into());
                Ok(ControlReply {
                    messages: vec![],
                    close_input: self.result_seen && state == "idle",
                })
            }
            Some("result") => {
                self.result_seen = true;
                Ok(ControlReply {
                    messages: vec![],
                    close_input: self.state.as_deref().is_none_or(|s| s == "idle"),
                })
            }
            Some(_) => Ok(ControlReply::default()),
            None => Err("native control message has no type".into()),
        }
    }
}
