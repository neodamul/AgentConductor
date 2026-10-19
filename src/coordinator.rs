use crate::{
    adapters::{self, InputMode},
    plan::AgentProfile,
    process::{self, ProcessSpec},
    protocol::Provider,
};
use serde_json::Value;

mod document;
pub use document::{failure, read_document, read_profile_document};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

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
        return read_only_args(&restricted, task_file);
    }
    let mut args = adapters::session_args(profile, None, None, task_file)?;
    if profile.permission_mode == "native_sandbox"
        && profile.parser == Provider::Codex
        && crate::executable::program_name(&profile.executable) == "codex"
    {
        if args != adapters::invocation(Provider::Codex, None, task_file)?.args {
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

pub async fn run_json(
    profile: &AgentProfile,
    cwd: &Path,
    dir: &Path,
    prompt: &str,
    field: &str,
    stop_file: Option<PathBuf>,
) -> Result<Value, String> {
    run_json_within(
        profile,
        cwd,
        dir,
        prompt,
        field,
        stop_file,
        Duration::from_secs(1800),
    )
    .await
}

pub async fn run_json_within(
    profile: &AgentProfile,
    cwd: &Path,
    dir: &Path,
    prompt: &str,
    field: &str,
    stop_file: Option<PathBuf>,
    timeout: Duration,
) -> Result<Value, String> {
    if prompt.len() > 128 * 1024 {
        return Err("coordinator prompt exceeds 128 KiB".into());
    }
    let task_file = dir.join("prompt.txt");
    let args = read_only_args(
        profile,
        task_file
            .to_str()
            .ok_or("non-UTF8 coordinator context path")?,
    )?;
    let args = if !cwd.join(".git").exists() {
        adapters::direct_folder_args(profile.parser, args)?
    } else {
        args
    };
    fs::create_dir(dir).map_err(|e| e.to_string())?;
    fs::write(task_file, prompt).map_err(|e| e.to_string())?;
    // A model "at capacity" is momentary (user decision 2026-10-14): ask again after a short
    // pause, at most three times, keeping each earlier attempt's logs.
    let mut retries = 0;
    loop {
        let result = process::run_until(
            ProcessSpec {
                executable: profile.executable.clone().into(),
                args: args.clone(),
                cwd: cwd.into(),
                input: if profile.input_mode == InputMode::Stdin {
                    prompt.as_bytes().to_vec()
                } else {
                    vec![]
                },
                log_dir: dir.join("process"),
                timeout,
                signal_grace: Duration::from_secs(5),
                environment: profile.environment.clone(),
            },
            stop_file.clone(),
        )
        .await?;
        let outcome = if result.exit_code != Some(0) || result.timed_out || result.stopped {
            Err(failure("coordinator", &result))
        } else {
            read_profile_document(profile, &result.stdout, field)
        };
        match outcome {
            Err(error)
                if retries < 3
                    && !result.stopped
                    && crate::availability::transient_capacity(&error) =>
            {
                retries += 1;
                fs::rename(
                    dir.join("process"),
                    dir.join(format!("process-capacity-{retries}")),
                )
                .map_err(|e| e.to_string())?;
                tokio::time::sleep(CAPACITY_RETRY_PAUSE).await;
            }
            outcome => return outcome,
        }
    }
}

/// Pause before the coordinator asks a model that was at capacity again.
const CAPACITY_RETRY_PAUSE: Duration = Duration::from_secs(30);
