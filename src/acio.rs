//! GitHub #208: running a CLI that speaks acio (docs/cli-io-protocol.md). The supervisor validates every event line,
//! asks the caller for a decision on each tool request that needs one (no answer in time means denied), and
//! writes `acio.control/1` lines (decisions, user messages, cancel) to the CLI in the order they were queued.
use crate::process::{ControlReply, ProcessResult, ProcessSpec};
use serde_json::{Value, json};
use std::{
    future::Future,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

const DEFAULT_DECISION_MS: u64 = 120_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
}

pub struct ToolRequest {
    pub call_id: String,
    pub name: String,
    pub input: Value,
}

/// The outgoing control lines of one run. Clones share one queue and one `seq` counter.
#[derive(Clone)]
pub struct Control {
    queue: Arc<Mutex<(u64, tokio::sync::mpsc::UnboundedSender<Vec<u8>>)>>,
    receiver: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>>>>,
}

impl Default for Control {
    fn default() -> Self {
        Self::new()
    }
}

impl Control {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            queue: Arc::new(Mutex::new((0, tx))),
            receiver: Arc::new(Mutex::new(Some(rx))),
        }
    }
    /// Numbering and queueing happen under one lock, so `seq` order is write order.
    fn send(&self, kind: &str, data: Value) -> Result<(), String> {
        let mut queue = self.queue.lock().map_err(|_| "control queue poisoned")?;
        queue.0 += 1;
        let line = json!({"schema":"acio.control/1","seq":queue.0,"type":kind,"data":data});
        crate::cli_io::validate("control", &line).map_err(|e| e.join("; "))?;
        queue
            .1
            .send(input_line(&line))
            .map_err(|_| "the run has ended".to_string())
    }
    pub fn user_message(&self, text: &str) -> Result<(), String> {
        self.send("user_message", json!({"text":text}))
    }
    /// Asks the CLI to stop at a safe point; the caller still stops the process after its own grace period.
    pub fn cancel(&self, reason: &str) -> Result<(), String> {
        self.send("cancel", json!({"reason":reason}))
    }
}

#[derive(Default)]
pub struct Options {
    pub control: Option<Control>,
    pub stop_file: Option<std::path::PathBuf>,
    pub paused: Option<Arc<std::sync::atomic::AtomicBool>>,
}

pub struct Run {
    pub result: ProcessResult,
}

/// One compact JSON document and a newline: the first stdin line, or a control line.
pub fn input_line(document: &Value) -> Vec<u8> {
    let mut line = document.to_string().into_bytes();
    line.push(b'\n');
    line
}

fn violation(
    message: &Value,
    expected: u64,
    finished: bool,
    session: Option<&str>,
) -> Result<(), String> {
    if let Err(errors) = crate::cli_io::validate("event", message) {
        return Err(format!("protocol violation: {}", errors[0]));
    }
    let seq = message["seq"].as_u64().unwrap_or(0);
    if seq != expected {
        return Err(format!(
            "protocol violation: seq {seq} should be {expected}"
        ));
    }
    if finished {
        return Err("protocol violation: event after the result".into());
    }
    if session.is_some_and(|id| message["session_id"] != id) {
        return Err(
            "protocol violation: event session_id does not match the assigned session".into(),
        );
    }
    Ok(())
}

/// Runs one CLI process under the acio contract. `decide` is asked for every tool request that needs a decision.
pub async fn run<F, Fut>(spec: ProcessSpec, decide: F, options: Options) -> Result<Run, String>
where
    F: Fn(ToolRequest) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Decision, String>> + Send + 'static,
{
    let first = spec.input.split(|b| *b == b'\n').next().unwrap_or(&[]);
    let input: Value =
        serde_json::from_slice(first).map_err(|e| format!("invalid acio input: {e}"))?;
    crate::cli_io::validate("input", &input)
        .map_err(|e| format!("invalid acio input: {}", e.join("; ")))?;
    let wait = Duration::from_millis(
        input["control"]["decision_timeout_ms"]
            .as_u64()
            .unwrap_or(DEFAULT_DECISION_MS),
    );
    let expected_session = input["session"]["id"].as_str().map(str::to_owned);
    let control = options.control.unwrap_or_default();
    let queue = control
        .receiver
        .lock()
        .map_err(|_| "control queue poisoned")?
        .take()
        .ok_or("this control channel already belongs to a run")?;
    let decide = Arc::new(decide);
    let sender = control.clone();
    let (mut expected, mut finished) = (1u64, false);
    let handler: crate::process::AsyncControlHandler = Box::new(move |message: Value| {
        let verdict = violation(&message, expected, finished, expected_session.as_deref());
        expected += 1;
        let ends = message["type"] == "result";
        finished |= ends;
        let (decide, sender) = (decide.clone(), sender.clone());
        Box::pin(async move {
            verdict?;
            let data = &message["data"];
            if message["type"] == "tool"
                && data["status"] == "requested"
                && data["needs_decision"] == true
            {
                let request = ToolRequest {
                    call_id: data["call_id"].as_str().unwrap_or("").into(),
                    name: data["name"].as_str().unwrap_or("").into(),
                    input: data["input"].clone(),
                };
                let call_id = request.call_id.clone();
                let decision = tokio::time::timeout(wait, decide(request))
                    .await
                    .map_err(|_| "no decision in time".to_string())??;
                let decision = match decision {
                    Decision::Allow => "allow",
                    Decision::Deny => "deny",
                };
                let line = json!({"call_id":call_id,"decision":decision});
                sender.send("tool_decision", line)?;
            }
            Ok(ControlReply {
                messages: vec![],
                close_input: ends,
            })
        })
    });
    let result = crate::process::run_controlled_outbound(
        spec,
        options.stop_file,
        handler,
        wait + Duration::from_secs(5),
        queue,
        options.paused,
    )
    .await?;
    Ok(Run { result })
}

/// Registers a CLI by what it says about itself: `<exe> --acio-describe` must print one valid
/// `acio.describe/1` document and exit 0 within `limit`.
pub async fn describe(executable: &Path, limit: Duration) -> Result<Value, String> {
    let exe = crate::executable::resolve(&executable.to_string_lossy())
        .unwrap_or_else(|| executable.to_path_buf());
    let mut command = tokio::process::Command::new(exe);
    command
        .arg("--acio-describe")
        .env_remove("AC_UI_EVENT_ADDRESS")
        .env_remove("AC_UI_EVENT_TOKEN")
        .env_remove("AC_UI_EVENT_SURFACE")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    let child = command
        .spawn()
        .map_err(|e| format!("spawn {}: {e}", executable.display()))?;
    let output = tokio::time::timeout(limit, child.wait_with_output())
        .await
        .map_err(|_| {
            format!(
                "{} did not answer --acio-describe within {} ms",
                executable.display(),
                limit.as_millis()
            )
        })?
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "{} --acio-describe exited with {}",
            executable.display(),
            output.status
        ));
    }
    if output.stdout.len() > 256 * 1024 {
        return Err("the describe document exceeds 256 KiB".into());
    }
    let document: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("the describe output is not JSON: {e}"))?;
    crate::cli_io::validate("describe", &document).map_err(|e| e.join("; "))?;
    Ok(document)
}
