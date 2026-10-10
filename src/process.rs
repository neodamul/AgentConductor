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
    run_with_control(spec, stop_file, None).await
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
    run_controlled_async_within(spec, stop_file, handler, Duration::from_secs(30)).await
}

/// Like `run_controlled_async`, but a single control evaluation may take `limit`,
/// e.g. while a paused tool call awaits a human decision.
pub async fn run_controlled_async_within(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
    handler: AsyncControlHandler,
    limit: Duration,
) -> Result<ProcessResult, String> {
    run_with_control(spec, stop_file, Some((handler, limit))).await
}

async fn run_with_control(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
    control: Option<(AsyncControlHandler, Duration)>,
) -> Result<ProcessResult, String> {
    use std::process::Stdio;
    use tokio::{fs::File, io::AsyncWriteExt, process::Command};

    if spec.timeout.is_zero() {
        return Err("process timeout must be positive".into());
    }
    // Never overwrite the evidence of an existing attempt.
    tokio::fs::create_dir(&spec.log_dir)
        .await
        .map_err(|e| format!("create attempt logs: {e}"))?;
    let stdout = spec.log_dir.join("stdout.jsonl");
    let stderr = spec.log_dir.join("stderr.log");
    let out_file = File::options()
        .write(true)
        .create_new(true)
        .open(&stdout)
        .await
        .map_err(|e| e.to_string())?;
    let err_file = File::options()
        .write(true)
        .create_new(true)
        .open(&stderr)
        .await
        .map_err(|e| e.to_string())?;
    let mut owner = Ownership {
        coordinator_pid: std::process::id(),
        pid: None,
        process_group: None,
        phase: "launching".into(),
        exit_code: None,
        termination_signal: None,
        stopped: false,
        timed_out: false,
    };
    record_owner(&spec.log_dir, &owner)?;
    if stop_file.as_ref().is_some_and(|p| p.exists()) {
        owner.phase = "not_spawned".into();
        owner.stopped = true;
        record_owner(&spec.log_dir, &owner)?;
        return Ok(ProcessResult {
            exit_code: None,
            termination_signal: None,
            timed_out: false,
            stopped: true,
            stdout,
            stderr,
        });
    }
    let mut command = Command::new(&spec.executable);
    command
        .args(&spec.args)
        .env_remove("AC_CHECKPOINT_RUN_DIR")
        .env_remove("AC_CHECKPOINT_ADDRESS_FILE")
        .env_remove("AC_CHECKPOINT_EXECUTABLE")
        .envs(&spec.environment)
        .current_dir(&spec.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    ProcessGroup::prepare(&mut command);
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            owner.phase = "not_spawned".into();
            record_owner(&spec.log_dir, &owner)?;
            return Err(format!("spawn agent: {error}"));
        }
    };
    let group = match ProcessGroup::adopt(&child) {
        Ok(group) => group,
        Err(error) => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            owner.phase = "not_spawned".into();
            record_owner(&spec.log_dir, &owner)?;
            return Err(error);
        }
    };
    owner.pid = child.id();
    owner.process_group = Some(group.id());
    owner.phase = "running".into();
    if let Err(error) = record_owner(&spec.log_dir, &owner) {
        drop(group);
        let _ = child.wait().await;
        return Err(error);
    }
    let mut input = child.stdin.take().ok_or("missing stdin")?;
    let mut output = child.stdout.take().ok_or("missing stdout")?;
    let mut errors = child.stderr.take().ok_or("missing stderr")?;
    let (fault_tx, fault_rx) = std::sync::mpsc::channel();
    let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let control_cancel = cancelled.clone();
    let exited = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let control_exit = exited.clone();
    let (input_task, out_task) = if let Some((handler, limit)) = control {
        (
            None,
            tokio::spawn(async move {
                let result = control_io(
                    output,
                    input,
                    out_file,
                    spec.input,
                    (handler, limit),
                    control_cancel,
                    control_exit,
                )
                .await;
                if let Err(error) = &result {
                    let _ = fault_tx.send(error.clone());
                }
                result
            }),
        )
    } else {
        (
            Some(tokio::spawn(async move {
                input.write_all(&spec.input).await?;
                input.shutdown().await
            })),
            tokio::spawn(async move {
                let _sender = fault_tx;
                let mut file = out_file;
                tokio::io::copy(&mut output, &mut file)
                    .await
                    .map_err(|e| e.to_string())?;
                file.flush().await.map_err(|e| e.to_string())
            }),
        )
    };
    let err_task = tokio::spawn(async move {
        let mut file = err_file;
        tokio::io::copy(&mut errors, &mut file).await?;
        file.flush().await
    });

    let stop_signal = async {
        match stop_file.clone() {
            Some(path) => loop {
                if tokio::fs::try_exists(&path).await? {
                    return Ok::<_, std::io::Error>(());
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            },
            None => std::future::pending().await,
        }
    };
    let mut stopped = false;
    let mut timed_out = false;
    let mut control_error = None;
    let fault_signal = async move {
        loop {
            match fault_rx.try_recv() {
                Ok(error) => return error,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => std::future::pending().await,
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    };
    let status = tokio::select! {
        biased;
        error=fault_signal=>{control_error=Some(error);None},
        status=child.wait()=>Some(status.map_err(|e|e.to_string())?),
        signal=stop_signal=>{signal.map_err(|e|e.to_string())?;stopped=true;None},
        _=tokio::time::sleep(spec.timeout)=>{timed_out=true;None},
    };
    if stop_file.as_ref().is_some_and(|path| path.exists()) {
        stopped = true;
    }
    // Cancel pending verification before signal escalation, not after its grace periods.
    cancelled.store(
        stopped || timed_out || control_error.is_some(),
        std::sync::atomic::Ordering::Release,
    );
    if status.is_some() {
        exited.store(true, std::sync::atomic::Ordering::Release);
    }
    let status = match status {
        Some(status) => status,
        None if control_error.is_some() => {
            group.kill();
            child.wait().await.map_err(|e| e.to_string())?
        }
        None => {
            group.interrupt();
            let status = match tokio::time::timeout(spec.signal_grace, child.wait()).await {
                Ok(status) => status,
                Err(_) => {
                    group.terminate();
                    match tokio::time::timeout(spec.signal_grace, child.wait()).await {
                        Ok(status) => status,
                        Err(_) => {
                            group.kill();
                            child.wait().await
                        }
                    }
                }
            };
            status.map_err(|e| e.to_string())?
        }
    };
    // Descendants must not outlive this attempt or keep its pipes open.
    drop(group);
    exited.store(true, std::sync::atomic::Ordering::Release);
    cancelled.store(
        stopped || timed_out || control_error.is_some(),
        std::sync::atomic::Ordering::Release,
    );
    owner.phase = "terminal".into();
    owner.exit_code = status.code();
    owner.termination_signal = exit_signal(&status);
    owner.stopped = stopped;
    owner.timed_out = timed_out;
    record_owner(&spec.log_dir, &owner)?;
    let input_result = match input_task {
        Some(task) => task.await.map_err(|e| e.to_string())?,
        None => Ok(()),
    };
    let output_result = out_task.await.map_err(|e| e.to_string())?;
    err_task
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    if let Some(error) = control_error.or_else(|| output_result.err()) {
        std::fs::write(spec.log_dir.join("control-error.txt"), &error)
            .map_err(|e| e.to_string())?;
        if !stopped && !timed_out {
            return Err(format!(
                "control channel failed; owned process group terminated: {error}"
            ));
        }
    }
    if status.success() && !timed_out && !stopped {
        input_result.map_err(|e| format!("write agent input: {e}"))?;
    }
    Ok(ProcessResult {
        exit_code: status.code(),
        termination_signal: exit_signal(&status),
        timed_out,
        stopped,
        stdout,
        stderr,
    })
}

mod control;
use control::*;
mod containment;
pub(crate) use containment::*;
#[cfg(windows)]
mod windows_job;
