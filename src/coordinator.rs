use crate::{
    adapters::{self, InputMode},
    plan::AgentProfile,
    process::{self, ProcessSpec},
    protocol::{Decoder, EventKind, Provider},
};
use serde_json::Value;
use std::{
    fs,
    io::Read,
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

/// The whole reply as JSON, or else the first fenced ```json block that carries `field`.
/// Models often wrap the requested object in a Markdown fence despite "return only JSON".
fn json_reply(text: &str, field: &str) -> Option<Value> {
    let has = |v: &Value| v.get(field).is_some();
    if let Ok(json) = serde_json::from_str::<Value>(text.trim()) {
        return Some(json).filter(has);
    }
    let mut rest = text;
    while let Some(open) = rest.find("```") {
        let after = &rest[open + 3..];
        let body_start = after.find('\n')? + 1;
        let body = &after[body_start..];
        let close = body.find("```")?;
        if let Ok(json) = serde_json::from_str::<Value>(body[..close].trim())
            && has(&json)
        {
            return Some(json);
        }
        rest = &body[close + 3..];
    }
    None
}

/// The last `limit` bytes of a log, as text (empty when unreadable).
fn log_tail(path: &Path, limit: u64) -> String {
    use std::io::{Seek, SeekFrom};
    let Ok(mut file) = fs::File::open(path) else {
        return String::new();
    };
    let len = file.metadata().map_or(0, |m| m.len());
    let _ = file.seek(SeekFrom::Start(len.saturating_sub(limit)));
    let mut bytes = vec![];
    let _ = file.take(limit).read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).trim().to_string()
}

/// The text of an error event, bounded, for the operator.
fn event_error(payload: &Value) -> String {
    let text = [
        "/api_error",
        "/error/message",
        "/message",
        "/error",
        "/result",
        "/item/message",
    ]
    .iter()
    .find_map(|p| payload.pointer(p).and_then(Value::as_str))
    .map(str::to_owned)
    .unwrap_or_else(|| payload.to_string());
    let text = if let Some(status) = payload["api_error_status"].as_i64() {
        format!("{text} (HTTP {status})")
    } else {
        text
    };
    text.chars().take(2000).collect()
}

/// Why a coordinator process failed: its status and the end of its own output, so the
/// operator sees the cause instead of only "logs retained".
pub fn failure(label: &str, result: &process::ProcessResult) -> String {
    let status = if result.stopped {
        "stopped".to_string()
    } else if result.timed_out {
        "timed out".to_string()
    } else {
        format!(
            "exit code {}",
            result.exit_code.map_or("none".into(), |c| c.to_string())
        )
    };
    let stderr = log_tail(&result.stderr, 1500);
    let stdout = log_tail(&result.stdout, 1500);
    let detail = if !stderr.is_empty() {
        stderr
    } else {
        // Without stderr, the last output line (often a JSON error event) carries the cause.
        stdout.lines().last().unwrap_or_default().to_string()
    };
    let logs = result.stdout.parent().unwrap_or(Path::new("")).display();
    if detail.is_empty() {
        format!("{label} failed ({status}); logs: {logs}")
    } else {
        format!("{label} failed ({status}): {detail}\nlogs: {logs}")
    }
}

pub fn read_document(provider: Provider, stdout: &Path, field: &str) -> Result<Value, String> {
    let mut decoder = Decoder::new(provider, 1024 * 1024);
    let mut file = fs::File::open(stdout).map_err(|e| e.to_string())?;
    let mut bytes = [0; 8192];
    let mut document = None;
    let mut text = String::new();
    let mut terminal = false;
    loop {
        let n = file.read(&mut bytes).map_err(|e| e.to_string())?;
        let batch = if n == 0 {
            decoder.finish()?
        } else {
            decoder.push(&bytes[..n])?
        };
        for event in batch {
            if event.kind == EventKind::Error {
                return Err(format!(
                    "coordinator reported an error: {}",
                    event_error(&event.payload)
                ));
            }
            terminal |= event.kind == EventKind::Result;
            let value = match provider {
                Provider::Claude if event.kind == EventKind::Result => event
                    .payload
                    .get("structured_output")
                    .or_else(|| event.payload.get("result")),
                Provider::Codex
                    if event.payload["type"] == "item.completed"
                        && event.payload.pointer("/item/type").and_then(Value::as_str)
                            == Some("agent_message") =>
                {
                    event.payload.pointer("/item/text")
                }
                Provider::OpenCode if event.payload["type"] == "text" => {
                    event.payload.pointer("/part/text")
                }
                Provider::Custom if event.kind == EventKind::Result => event.payload.get("payload"),
                _ => None,
            };
            if let Some(value) = value {
                if let Some(chunk) = value.as_str() {
                    if chunk.len() > 64 * 1024 {
                        return Err("coordinator document exceeds 64 KiB".into());
                    }
                    if provider == Provider::OpenCode {
                        if text.len().saturating_add(chunk.len()) > 64 * 1024 {
                            return Err("coordinator document exceeds 64 KiB".into());
                        }
                        text.push_str(chunk);
                    } else if let Some(json) = json_reply(chunk, field) {
                        document = Some(json);
                    }
                } else if value.get(field).is_some() {
                    if serde_json::to_vec(value).map_err(|e| e.to_string())?.len() > 64 * 1024 {
                        return Err("coordinator document exceeds 64 KiB".into());
                    }
                    document = Some(value.clone());
                }
            }
        }
        if n == 0 {
            break;
        }
    }
    if provider == Provider::OpenCode && !text.is_empty() {
        document = json_reply(&text, field);
        terminal = document.is_some();
    }
    if !terminal {
        return Err("coordinator did not return successful final output".into());
    }
    document.ok_or_else(|| format!("coordinator response contains no valid JSON {field} document"))
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
    let result = process::run_until(
        ProcessSpec {
            executable: profile.executable.clone().into(),
            args,
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
        stop_file,
    )
    .await?;
    if result.exit_code != Some(0) || result.timed_out || result.stopped {
        return Err(failure("coordinator", &result));
    }
    read_document(profile.parser, &result.stdout, field)
}
