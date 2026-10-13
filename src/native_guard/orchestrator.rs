//! The run's orchestrator decides tool requests the workspace sandbox cannot permit.
use super::{Binding, random_bytes};
use crate::{claude_control::ToolDecision, gates::ToolRequest, plan::AgentProfile};
use std::path::{Path, PathBuf};

/// Seconds the orchestrator has to decide one tool request (two attempts share them).
pub(super) const ORCHESTRATOR_WAIT: u64 = 300;

/// The workspace's coordinator profile, written beside the run by the desktop.
pub fn orchestrator_path(run: &Path) -> PathBuf {
    crate::messenger::sibling(run, "orchestrator.json")
}

/// The orchestrator: the profile written beside the run, or for a run started before that
/// file existed, the coordinator of the workspace that holds the run.
fn profile(run: &Path) -> Option<AgentProfile> {
    if let Some(profile) = std::fs::read(orchestrator_path(run))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    {
        return Some(profile);
    }
    let settings: serde_json::Value =
        serde_json::from_slice(&std::fs::read(run.parent()?.join("startup.json")).ok()?).ok()?;
    let id = settings["config"]["coordinator"].as_str()?;
    let profile = settings["config"]["profiles"]
        .as_array()?
        .iter()
        .find(|p| p["id"] == id)?;
    serde_json::from_value(profile.clone()).ok()
}

fn clip(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_owned();
    }
    let mut end = limit;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &text[..end])
}

/// Asks the run's orchestrator (the workspace coordinator) to decide a request the
/// workspace policy cannot permit by itself. User decisions (2026-10-13): the orchestrator
/// decides routine requests itself and every decision is recorded with its request and
/// reason; a person is asked only when no orchestrator gives a valid answer.
pub(super) async fn orchestrate(
    binding: &Binding,
    request: &ToolRequest,
    stop: &Path,
) -> Option<ToolDecision> {
    let profile = profile(&binding.dir)?;
    let report = crate::workflow::load(&binding.dir).ok()?;
    let task = report.plan.tasks.iter().find(|t| t.id == binding.task)?;
    let input = clip(&request.input.to_string(), 16 * 1024);
    let prompt = format!(
        "You are the orchestrator of a multi-agent development run. A worker agent asked to use a \
         tool in a way the workspace sandbox does not permit on its own; decide now so the work \
         keeps moving. Prefer allow: reads, builds, tests, scratch files in temporary folders, \
         dependency or documentation downloads and other routine development steps are allowed. \
         Deny only when the request publishes anything (push, deploy, release, messages), deletes \
         or changes data outside the project, exposes secrets or credentials, or changes system \
         settings. The worker continues either way and can choose another action.\n\n\
         Run goal:\n{}\n\nTask {} (owned paths: {}):\n{}\n\nTool: {}\nInput: {}\n\n\
         Return only JSON: {{\"decision\":\"allow\" or \"deny\",\"reason\":\"one sentence\"}}",
        clip(&report.plan.goal, 8 * 1024),
        task.id,
        task.paths.join(", "),
        clip(&task.prompt, 32 * 1024),
        request.tool,
        input,
    );
    // One retry when the orchestrator gives no valid answer.
    let mut attempts = 0;
    let (dir, reply, decision) = loop {
        attempts += 1;
        let mut nonce = [0u8; 6];
        random_bytes(&mut nonce).ok()?;
        let nonce: String = nonce.iter().map(|b| format!("{b:02x}")).collect();
        let dir = binding.dir.join(format!(
            "orchestrator-{}-{nonce}",
            binding.task.replace(['/', '\\'], "_")
        ));
        let reply = crate::coordinator::run_json_within(
            &profile,
            &binding.dir,
            &dir,
            &prompt,
            "decision",
            Some(stop.to_path_buf()),
            std::time::Duration::from_secs(ORCHESTRATOR_WAIT / 2),
        )
        .await;
        let decision = read_decision(&reply, &profile.id);
        if decision.is_some() || attempts >= 2 || stop.exists() {
            break (dir, reply, decision);
        }
    };
    if let Ok(mut store) = crate::state::Store::open(binding.dir.join("state.sqlite")) {
        let _ = store.record(
            "tool_orchestrator_decision",
            &serde_json::json!({
                "task": binding.task,
                "orchestrator": profile.id,
                "tool": request.tool,
                "input": clip(&input, 2048),
                "allow": decision.as_ref().map(|d| d.allow),
                "reason": decision.as_ref().map(|d| d.reason.clone()),
                "error": reply.as_ref().err(),
                "attempts": attempts,
                "decided_ms": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or_default(),
                "artifacts": dir,
            }),
            match &decision {
                Some(d) if d.allow => {
                    "Orchestrator allowed a tool request outside the workspace sandbox"
                }
                Some(_) => "Orchestrator denied a tool request outside the workspace sandbox",
                None => "Orchestrator gave no decision; a person is asked",
            },
        );
    }
    decision
}

fn read_decision(
    reply: &Result<serde_json::Value, String>,
    orchestrator: &str,
) -> Option<ToolDecision> {
    let reply = reply.as_ref().ok()?;
    let reason = reply["reason"].as_str().unwrap_or("").trim();
    let reason = clip(
        if reason.is_empty() {
            "no reason given"
        } else {
            reason
        },
        1024,
    );
    match reply["decision"].as_str() {
        Some("allow") => Some(ToolDecision {
            allow: true,
            reason: format!("Orchestrator {orchestrator} allowed: {reason}"),
        }),
        Some("deny") => Some(ToolDecision::deny(&format!(
            "Orchestrator {orchestrator} denied: {reason}"
        ))),
        _ => None,
    }
}
