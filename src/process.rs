use std::{path::PathBuf, time::Duration};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Ownership {
    pub coordinator_pid: u32,
    pub pid: Option<u32>,
    pub process_group: Option<i32>,
    pub phase: String,
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub termination_signal: Option<i32>,
    pub stopped: bool,
    pub timed_out: bool,
}

impl Ownership {
    #[cfg(unix)]
    pub fn ensure_quiescent(&self) -> Result<(), String> {
        if self.phase == "not_spawned" && self.pid.is_none() && self.process_group.is_none() {
            return Ok(());
        }
        if !matches!(self.phase.as_str(), "running" | "terminal") {
            return Err("ambiguous process ownership; manual investigation required".into());
        }
        let pid = self
            .pid
            .filter(|id| *id > 0 && *id <= i32::MAX as u32)
            .ok_or("invalid or missing owned PID")? as i32;
        if self.process_group != Some(pid) {
            return Err("invalid or missing owned process group".into());
        }
        for target in [pid, -pid] {
            // SAFETY: kill with signal 0 only probes for existence; `target` is a validated PID or its negation and no memory is passed.
            if unsafe { libc::kill(target, 0) } == 0 {
                return Err("owned process or group is still present; recovery refused".into());
            }
            if std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
                return Err("cannot prove owned process absence; recovery refused".into());
            }
        }
        Ok(())
    }

    #[cfg(windows)]
    pub fn ensure_quiescent(&self) -> Result<(), String> {
        if self.phase == "not_spawned" && self.pid.is_none() && self.process_group.is_none() {
            return Ok(());
        }
        if !matches!(self.phase.as_str(), "running" | "terminal") {
            return Err("ambiguous process ownership; manual investigation required".into());
        }
        let pid = self
            .pid
            .filter(|id| *id > 0 && *id <= i32::MAX as u32)
            .ok_or("invalid or missing owned PID")?;
        if self.process_group != Some(pid as i32) {
            return Err("invalid or missing owned process group".into());
        }
        // Job members die with the job handle, so the root's absence proves quiescence.
        if !windows_job::process_absent(pid)? {
            return Err("owned process or group is still present; recovery refused".into());
        }
        Ok(())
    }
}

pub(crate) fn record_owner(dir: &std::path::Path, owner: &Ownership) -> Result<(), String> {
    use std::io::Write;
    let pending = dir.join("owner.pending");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending)
        .map_err(|e| format!("create process ownership record: {e}"))?;
    file.write_all(&serde_json::to_vec(owner).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    std::fs::rename(pending, dir.join("owner.json")).map_err(|e| e.to_string())?;
    sync_directory(dir)
}

pub struct ProcessSpec {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub input: Vec<u8>,
    pub log_dir: PathBuf,
    pub timeout: Duration,
    pub signal_grace: Duration,
    pub environment: std::collections::BTreeMap<String, String>,
}

pub struct ProcessResult {
    pub exit_code: Option<i32>,
    pub termination_signal: Option<i32>,
    pub timed_out: bool,
    pub stopped: bool,
    pub stdout: PathBuf,
    pub stderr: PathBuf,
}

#[derive(Default)]
pub struct ControlReply {
    pub messages: Vec<serde_json::Value>,
    pub close_input: bool,
}

pub type ControlHandler = Box<dyn FnMut(serde_json::Value) -> Result<ControlReply, String> + Send>;
pub type AsyncControlHandler = Box<
    dyn FnMut(
            serde_json::Value,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<ControlReply, String>> + Send>,
        > + Send,
>;

pub async fn run(spec: ProcessSpec) -> Result<ProcessResult, String> {
    run_until(spec, None).await
}

pub async fn run_until(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
) -> Result<ProcessResult, String> {
    runner::run_with_control(spec, stop_file, None, None, None).await
}

pub async fn run_controlled(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
    mut handler: ControlHandler,
) -> Result<ProcessResult, String> {
    run_controlled_async(
        spec,
        stop_file,
        Box::new(move |message| {
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(message)))
                    .map_err(|_| "control evaluator panicked".to_string())
                    .and_then(|result| result);
            Box::pin(async move { result })
        }),
    )
    .await
}

pub async fn run_controlled_async(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
    handler: AsyncControlHandler,
) -> Result<ProcessResult, String> {
    run_controlled_async_within(spec, stop_file, handler, Duration::from_secs(30), None).await
}

/// Like `run_controlled_async`, but a single control evaluation may take `limit`,
/// e.g. while a paused tool call awaits a human decision. While `paused` is set the
/// process timeout does not run down, so a slow decision never times the task out.
pub async fn run_controlled_async_within(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
    handler: AsyncControlHandler,
    limit: Duration,
    paused: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
) -> Result<ProcessResult, String> {
    runner::run_with_control(spec, stop_file, Some((handler, limit)), paused, None).await
}

pub async fn run_controlled_outbound(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
    handler: AsyncControlHandler,
    limit: Duration,
    outbound: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
) -> Result<ProcessResult, String> {
    runner::run_with_control(
        spec,
        stop_file,
        Some((handler, limit)),
        None,
        Some(outbound),
    )
    .await
}

mod control;
mod runner;
use control::*;
mod containment;
pub(crate) use containment::*;
#[cfg(windows)]
pub(crate) mod windows_job;
