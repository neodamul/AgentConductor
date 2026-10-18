use crate::{
    claude_control::{Controller, ToolDecision},
    gates::ToolRequest,
    guard::{Disposition, Policy},
    plan::AgentProfile,
    process::{self, ProcessResult, ProcessSpec},
    protocol::Provider,
};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

mod decision;
mod orchestrator;
use decision::coordinated_decision;
use orchestrator::ORCHESTRATOR_WAIT;
pub use orchestrator::{orchestrator_path, orchestrator_profile};

pub fn validate(profile: &AgentProfile) -> Result<(), String> {
    if profile.parser != Provider::Claude
        || crate::executable::program_name(&profile.executable) != "claude"
        || profile.input_mode != crate::adapters::InputMode::Stdin
        || profile.args != crate::adapters::invocation(Provider::Claude, None, "unused")?.args
    {
        return Err(
            "native_callback requires the pinned Claude stdin/print/stream-json invocation".into(),
        );
    }
    Ok(())
}

pub fn session_id() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    random_bytes(&mut bytes)?;
    bytes[6] = (bytes[6] & 15) | 64;
    bytes[8] = (bytes[8] & 63) | 128;
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    ))
}

pub fn args(
    profile: &AgentProfile,
    model: Option<&str>,
    session: &str,
    resume: bool,
) -> Result<Vec<String>, String> {
    validate(profile)?;
    let mut args = crate::adapters::session_args(
        profile,
        model,
        if resume { Some(session) } else { None },
        "unused",
    )?;
    args.extend(
        [
            "--input-format",
            "stream-json",
            "--restricted",
            "--permission-mode",
            "manual",
            "--permission-prompts",
            "host",
            "--strict-mcp-config",
            "--mcp-config",
            "{\"mcpServers\":{}}",
            "--no-chrome",
            "--disable-slash-commands",
            "--tools",
            "Read,Edit,Write,Grep,Glob,Bash",
        ]
        .map(String::from),
    );
    if !resume {
        args.extend(["--session-id".into(), session.into()]);
    }
    Ok(args)
}

pub struct Outcome {
    pub process: Result<ProcessResult, String>,
    pub pending: Option<ToolRequest>,
    pub established: bool,
}

#[derive(Clone)]
pub struct Binding {
    pub dir: PathBuf,
    pub task: String,
    pub approved: Option<crate::gates::Receipt>,
}

pub async fn run(
    mut spec: ProcessSpec,
    stop: PathBuf,
    prompt: &str,
    session: &str,
    scopes: &[String],
    binding: Binding,
) -> Outcome {
    let pending: Arc<Mutex<Option<ToolRequest>>> = Arc::new(Mutex::new(None));
    let established = Arc::new(AtomicBool::new(false));
    // Time spent waiting for a human decision does not count toward the task timeout.
    let deciding = Arc::new(AtomicBool::new(false));
    let result = async {
        let policy = Arc::new(Policy::new(&spec.cwd, scopes)?);
        let slot: Arc<Mutex<Option<ToolDecision>>> = Arc::new(Mutex::new(None));
        let evaluated = slot.clone();
        let controller = Controller::for_session(
            prompt,
            session,
            Box::new(move |_, _| {
                evaluated
                    .lock()
                    .map_err(|_| "decision state poisoned")?
                    .take()
                    .ok_or_else(|| "Missing validated asynchronous decision".into())
            }),
        )?
        .with_hook_timeout(hook_timeout());
        spec.input = controller.initial_input()?;
        let controller = Arc::new(Mutex::new(controller));
        let expected = session.to_owned();
        let seen = established.clone();
        let waiting = pending.clone();
        let paused = deciding.clone();
        process::run_controlled_async_within(
            spec,
            Some(stop.clone()),
            Box::new(move |message| {
                let controller = controller.clone();
                let policy = policy.clone();
                let slot = slot.clone();
                let expected = expected.clone();
                let seen = seen.clone();
                let waiting = waiting.clone();
                let binding = binding.clone();
                let stop = stop.clone();
                let paused = paused.clone();
                Box::pin(async move {
                    let pre = controller
                        .lock()
                        .map_err(|_| "native controller state poisoned")?
                        .pretool(&message)?;
                    if let Some((tool, input)) = pre {
                        // An input the policy refuses is denied for this call only; the
                        // worker continues and can choose another action.
                        let disposition = policy
                            .classify(&tool, &input)
                            .unwrap_or_else(Disposition::Deny);
                        let decision = match disposition {
                            Disposition::Automatic => ToolDecision {
                                allow: true,
                                reason: "Workspace sandbox policy permits the request".into(),
                            },
                            Disposition::Deny(reason) => ToolDecision::deny(&reason),
                            Disposition::Human => {
                                let request = ToolRequest {
                                    tool,
                                    input,
                                    reason: format!(
                                        "{} requires an exact human decision",
                                        message["request"]["input"]["tool_name"].as_str().unwrap()
                                    ),
                                };
                                let mut allowed = false;
                                if let Some(approved) = &binding.approved
                                    && approved.request == request
                                {
                                    let context = crate::workflow::verify_tool_context(
                                        &binding.dir,
                                        &binding.task,
                                        &request,
                                        &std::env::current_exe().map_err(|e| e.to_string())?,
                                    )
                                    .await?;
                                    if !stop.exists() && context == approved.context {
                                        let now = std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs()
                                            as i64;
                                        allowed = crate::gates::Ledger::open(
                                            &binding.dir.join("state.sqlite"),
                                        )?
                                        .consume(&approved.id, &context, &request, now)?
                                            == crate::gates::Decision::Allowed;
                                    }
                                }
                                if allowed {
                                    ToolDecision {
                                        allow: true,
                                        reason: "Exact current human approval consumed once".into(),
                                    }
                                } else if let Some(decision) = {
                                    paused.store(true, Ordering::Release);
                                    let decision = coordinated_decision(
                                        &binding,
                                        &request,
                                        tool_use_id(&message)?,
                                        &expected,
                                        &stop,
                                    )
                                    .await;
                                    paused.store(false, Ordering::Release);
                                    decision?
                                } {
                                    decision
                                } else {
                                    *waiting.lock().map_err(|_| "pending tool state poisoned")? =
                                        Some(request);
                                    return Err(
                                        "Tool requires human approval; stop before execution"
                                            .into(),
                                    );
                                }
                            }
                        };
                        *slot.lock().map_err(|_| "decision state poisoned")? = Some(decision);
                    }
                    if stop.exists() {
                        return Err("Human stop prohibits a native control reply".into());
                    }
                    let session_seen = message["session_id"] == expected
                        || message["request"]["input"]["session_id"] == expected;
                    let reply = controller
                        .lock()
                        .map_err(|_| "native controller state poisoned")?
                        .handle(message)?;
                    if session_seen {
                        seen.store(true, Ordering::Release);
                    }
                    Ok(reply)
                })
            }),
            std::time::Duration::from_secs(u64::from(hook_timeout())),
            Some(deciding),
        )
        .await
    }
    .await;
    // A valid pre-tool callback can interrupt evaluation before Controller::handle returns.
    let request = pending.lock().ok().and_then(|value| value.clone());
    Outcome {
        process: result,
        established: established.load(Ordering::Acquire) || request.is_some(),
        pending: request,
    }
}

/// The CLI's identifier of the tool call a pre-tool callback is about.
fn tool_use_id(message: &serde_json::Value) -> Result<&str, String> {
    message["request"]["tool_use_id"]
        .as_str()
        .or_else(|| message["request"]["input"]["tool_use_id"].as_str())
        .filter(|id| !id.is_empty() && id.len() <= 256)
        .ok_or_else(|| "native tool-use identity missing".to_string())
}

/// Seconds a paused worker waits for a live human tool decision; 0 stops it instead.
pub(crate) fn decision_wait() -> u64 {
    std::env::var("AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(600)
        .min(3000)
}

fn hook_timeout() -> u16 {
    match decision_wait() {
        // A stored approval is re-verified (up to 30 s) before it is consumed.
        0 => (ORCHESTRATOR_WAIT + 45) as u16,
        // Leave margin for context verification after the human decides.
        wait => (ORCHESTRATOR_WAIT + wait + 60) as u16,
    }
}

/// Operating-system CSPRNG bytes (DD-37).
fn random_bytes(bytes: &mut [u8]) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::io::Read;
        std::fs::File::open("/dev/urandom")
            .and_then(|mut file| file.read_exact(bytes))
            .map_err(|e| e.to_string())
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::Security::Cryptography::{
            BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom,
        };
        let length = u32::try_from(bytes.len()).map_err(|_| "random request too large")?;
        let status = unsafe {
            BCryptGenRandom(
                std::ptr::null_mut(),
                bytes.as_mut_ptr(),
                length,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        };
        if status < 0 {
            return Err(format!("system random generator failed: {status:#x}"));
        }
        Ok(())
    }
}
