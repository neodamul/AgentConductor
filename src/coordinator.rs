use crate::{adapters, plan::AgentProfile, protocol::Provider};
use serde_json::Value;

mod document;
mod run;
pub use document::{failure, read_document, read_profile_document};
pub use run::{run_json, run_json_with_model, run_json_within};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Accept,
    Revise,
    Blocked,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revision {
    pub task: String,
    pub feedback: String,
    /// A replacement test command, only when the evidence shows the planned one cannot work
    /// in this project (for example git in a folder that is not a Git repository).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_command: Option<Vec<String>>,
    /// New task paths requested by the reviewer. They still pass the plan's scope rules.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub add_paths: Vec<String>,
    /// Existing task IDs that must complete before this task can run.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub add_dependencies: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewProposal {
    pub verdict: Verdict,
    pub summary: String,
    pub evidence: Vec<String>,
    pub revisions: Vec<Revision>,
}

impl ReviewProposal {
    pub fn validate(
        &self,
        tasks: &[String],
        evidence: &[String],
        verified: bool,
    ) -> Result<(), String> {
        if self.summary.trim().is_empty() || self.summary.len() > 8192 {
            return Err("review summary must contain 1..8192 bytes".into());
        }
        if self.evidence.is_empty()
            || self.evidence.len() > 12
            || self.evidence.iter().any(|id| !evidence.contains(id))
        {
            return Err("review must cite known evidence IDs".into());
        }
        if self.revisions.len() > 3 {
            return Err("review can revise at most three existing tasks".into());
        }
        let mut used = std::collections::HashSet::new();
        for revision in &self.revisions {
            if !tasks.contains(&revision.task)
                || !used.insert(&revision.task)
                || revision.feedback.trim().is_empty()
                || revision.feedback.len() > 16 * 1024
            {
                return Err("invalid, duplicate or unknown review task/feedback".into());
            }
            if !crate::workflow::guardrail::bounded(&revision.add_paths, &revision.add_dependencies)
                || revision.add_paths.iter().any(|p| p.trim().is_empty())
                || revision
                    .add_dependencies
                    .iter()
                    .any(|d| d.trim().is_empty())
            {
                return Err("amendment paths or dependencies exceed their bounds".into());
            }
            if let Some(command) = &revision.test_command
                && (command.is_empty()
                    || command.len() > 32
                    || command[0].trim().is_empty()
                    || command.iter().any(|a| a.len() > 4096 || a.contains('\0')))
            {
                return Err(
                    "a revised test command needs an executable and bounded arguments".into(),
                );
            }
        }
        match self.verdict {
            Verdict::Accept if !verified || !self.revisions.is_empty() => Err(
                "accept requires a verified candidate and no revisions; it is not human approval"
                    .into(),
            ),
            Verdict::Revise if self.revisions.is_empty() => {
                Err("revise requires at least one task revision".into())
            }
            Verdict::Blocked if !self.revisions.is_empty() => {
                Err("blocked cannot automatically revise tasks".into())
            }
            _ => Ok(()),
        }
    }
}

pub fn read_only_args(profile: &AgentProfile, task_file: &str) -> Result<Vec<String>, String> {
    read_only_args_with_model(profile, None, task_file)
}

pub fn read_only_args_with_model(
    profile: &AgentProfile,
    model: Option<&str>,
    task_file: &str,
) -> Result<Vec<String>, String> {
    // DD-131: a full-access agent still coordinates read-only, through its CLI's verified
    // sandboxed template restricted below.
    if profile.permission_mode == "full_access"
        && adapters::full_access_args(profile.parser).as_ref() == Some(&profile.args)
    {
        let mut restricted = profile.clone();
        restricted.args = adapters::invocation(profile.parser, None, "{task_file}")?.args;
        restricted.permission_mode = match profile.parser {
            Provider::Codex => "native_sandbox",
            _ => "native_callback",
        }
        .into();
        return read_only_args_with_model(&restricted, model, task_file);
    }
    let mut args = adapters::session_args(profile, model, None, task_file)?;
    if profile.permission_mode == "native_sandbox"
        && profile.parser == Provider::Codex
        && crate::executable::program_name(&profile.executable) == "codex"
    {
        if profile.args != adapters::invocation(Provider::Codex, None, task_file)?.args {
            return Err("coordinator requires the verified native Codex template".into());
        }
        args[4] = "read-only".into();
    } else if profile.permission_mode == "native_callback" {
        // Reuse the pinned guarded Claude template, exposing only read-only tools.
        crate::native_guard::validate(profile)?;
        args.extend(
            [
                "--tools",
                "Read,Grep,Glob",
                "--strict-mcp-config",
                "--mcp-config",
                "{\"mcpServers\":{}}",
                "--no-chrome",
                "--disable-slash-commands",
            ]
            .map(String::from),
        );
    } else if profile.permission_mode != "read_only" {
        return Err("coordinator requires read-only capabilities".into());
    }
    Ok(args)
}

/// Planning runs in a disposable snapshot, so a coordinator whose read-only behavior
/// cannot be verified may plan; verified templates keep their own restrictions.
pub fn planning_args(profile: &AgentProfile, task_file: &str) -> Result<Vec<String>, String> {
    read_only_args(profile, task_file)
        .or_else(|_| adapters::session_args(profile, None, None, task_file))
}
