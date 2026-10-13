//! The run's orchestrator decides tool requests the workspace sandbox cannot permit.
use super::{Binding, random_bytes};
use crate::{claude_control::ToolDecision, gates::ToolRequest, plan::AgentProfile};
use std::path::PathBuf;

/// Seconds the orchestrator has to decide one tool request.
pub(super) const ORCHESTRATOR_WAIT: u64 = 300;

/// The workspace's coordinator profile, written beside the run by the desktop.
pub fn orchestrator_path(run: &std::path::Path) -> PathBuf {
    crate::messenger::sibling(run, "orchestrator.json")
}

/// Asks the run's orchestrator (the workspace coordinator) to decide a request the
/// workspace policy cannot permit by itself. None when it is not configured or gives no
/// valid answer, so the caller asks a person instead.
pub(super) async fn orchestrate(
    binding: &Binding,
    request: &ToolRequest,
    stop: &std::path::Path,
) -> Option<ToolDecision> {
    let profile: AgentProfile = std::fs::read(orchestrator_path(&binding.dir))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())?;
    let report = crate::workflow::load(&binding.dir).ok()?;
    let task = report.plan.tasks.iter().find(|t| t.id == binding.task)?;
    let clip = |text: &str, limit: usize| -> String {
        if text.len() <= limit {
            return text.to_owned();
        }
        let mut end = limit;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &text[..end])
    };
    let input = clip(&request.input.to_string(), 16 * 1024);
    let prompt = format!(
        "You are the orchestrator of a multi-agent development run. A worker agent asked to use a \
         tool in a way the workspace sandbox does not permit on its own, so you decide.\n\
         Allow the request when the task plausibly needs it and it cannot harm the user's \
         machine, accounts or data beyond the project. Deny it when it is unrelated to the task, \
         destructive, exfiltrates secrets, publishes anything (push, deploy, release) or \
         changes the system. The worker continues either way and can choose another action.\n\n\
         Run goal:\n{}\n\nTask {} (owned paths: {}):\n{}\n\nTool: {}\nInput: {}\n\n\
         Return only JSON: {{\"decision\":\"allow\" or \"deny\",\"reason\":\"one sentence\"}}",
        clip(&report.plan.goal, 8 * 1024),
        task.id,
        task.paths.join(", "),
        clip(&task.prompt, 32 * 1024),
        request.tool,
        input,
    );
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
        std::time::Duration::from_secs(ORCHESTRATOR_WAIT),
    )
    .await;
    let decision = reply.as_ref().ok().and_then(|reply| {
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
                reason: format!("Orchestrator {} allowed: {reason}", profile.id),
            }),
            Some("deny") => Some(ToolDecision::deny(&format!(
                "Orchestrator {} denied: {reason}",
                profile.id
            ))),
            _ => None,
        }
    });
    if let Ok(mut store) = crate::state::Store::open(binding.dir.join("state.sqlite")) {
        let _ = store.record(
            "tool_orchestrator_decision",
            &serde_json::json!({
                "task": binding.task,
                "orchestrator": profile.id,
                "tool": request.tool,
                "allow": decision.as_ref().map(|d| d.allow),
                "reason": decision.as_ref().map(|d| d.reason.clone()),
                "error": reply.as_ref().err(),
                "artifacts": dir,
            }),
            match &decision {
                Some(_) => "Orchestrator decided a tool request outside the workspace sandbox",
                None => "Orchestrator gave no decision; a person is asked",
            },
        );
    }
    decision
}
