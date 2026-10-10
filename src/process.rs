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

    #[cfg(not(unix))]
    pub fn ensure_quiescent(&self) -> Result<(), String> {
        Err("process ownership verification is currently supported only on Unix".into())
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
    std::fs::File::open(dir)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())
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

struct AbortOnDrop<T>(tokio::task::JoinHandle<T>);
impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[cfg(unix)]
async fn control_io(
    mut output: tokio::process::ChildStdout,
    input: tokio::process::ChildStdin,
    mut file: tokio::fs::File,
    initial: Vec<u8>,
    mut handler: AsyncControlHandler,
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
    exited: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut input = Some(input);
    input
        .as_mut()
        .unwrap()
        .write_all(&initial)
        .await
        .map_err(|e| e.to_string())?;
    input
        .as_mut()
        .unwrap()
        .flush()
        .await
        .map_err(|e| e.to_string())?;
    let mut pending = Vec::new();
    let mut bytes = [0; 8192];
    loop {
        let n = output.read(&mut bytes).await.map_err(|e| e.to_string())?;
        if n == 0 {
            if !pending.is_empty() || input.is_some() {
                return Err(
                    "control stream ended before an explicit input close or with incomplete JSONL"
                        .into(),
                );
            }
            break;
        }
        // Evidence is persisted before evaluating any control message.
        file.write_all(&bytes[..n])
            .await
            .map_err(|e| e.to_string())?;
        file.flush().await.map_err(|e| e.to_string())?;
        pending.extend_from_slice(&bytes[..n]);
        while let Some(end) = pending.iter().position(|b| *b == b'\n') {
            if end > 1024 * 1024 {
                return Err("control JSONL message exceeds 1 MiB".into());
            }
            let message: serde_json::Value = serde_json::from_slice(&pending[..end])
                .map_err(|e| format!("invalid control JSONL: {e}"))?;
            pending.drain(..=end);
            let permission = message["type"] == "control_request";
            let evaluation =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(message)))
                    .map_err(|_| "control evaluator panicked")?;
            let mut evaluation = AbortOnDrop(tokio::spawn(evaluation));
            let cancel = async {
                while !cancelled.load(std::sync::atomic::Ordering::Acquire)
                    && !(permission && exited.load(std::sync::atomic::Ordering::Acquire))
                {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            };
            let reply = tokio::select! {
                biased;
                _=cancel=>return Err("control evaluation cancelled with the owned process".into()),
                result=&mut evaluation.0=>result.map_err(|_| "control evaluator panicked")??,
                _=tokio::time::sleep(Duration::from_secs(30))=>return Err("control evaluation exceeded 30 seconds".into()),
            };
            if cancelled.load(std::sync::atomic::Ordering::Acquire)
                || (permission && exited.load(std::sync::atomic::Ordering::Acquire))
            {
                return Err(
                    "control reply refused after process termination or cancellation".into(),
                );
            }
            if reply.messages.len() > 8 {
                return Err("control response exceeds eight messages".into());
            }
            let mut response = vec![];
            for value in reply.messages {
                if !value.is_object() {
                    return Err("control response must be a JSON object".into());
                }
                serde_json::to_writer(&mut response, &value).map_err(|e| e.to_string())?;
                response.push(b'\n');
                if response.len() > 256 * 1024 {
                    return Err("control response exceeds 256 KiB".into());
                }
            }
            if !response.is_empty() {
                let stdin = input
                    .as_mut()
                    .ok_or("control reply attempted after input was closed")?;
                stdin
                    .write_all(&response)
                    .await
                    .map_err(|e| e.to_string())?;
                stdin.flush().await.map_err(|e| e.to_string())?;
            }
            if reply.close_input
                && let Some(mut stdin) = input.take()
            {
                stdin.shutdown().await.map_err(|e| e.to_string())?;
            }
        }
        if pending.len() > 1024 * 1024 {
            return Err("control JSONL message exceeds 1 MiB".into());
        }
    }
    file.flush().await.map_err(|e| e.to_string())
}

#[cfg(unix)]
pub(crate) struct ProcessGroup(pub(crate) i32);

#[cfg(unix)]
impl ProcessGroup {
    fn signal(&self, signal: i32) {
        // The child starts a new process group whose ID equals its PID.
        unsafe { libc::kill(-self.0, signal) };
    }
}

#[cfg(unix)]
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        self.signal(libc::SIGKILL);
    }
}

#[cfg(unix)]
pub async fn run(spec: ProcessSpec) -> Result<ProcessResult, String> {
    run_until(spec, None).await
}

#[cfg(unix)]
pub async fn run_until(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
) -> Result<ProcessResult, String> {
    run_with_control(spec, stop_file, None).await
}

#[cfg(unix)]
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

#[cfg(unix)]
pub async fn run_controlled_async(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
    handler: AsyncControlHandler,
) -> Result<ProcessResult, String> {
    run_with_control(spec, stop_file, Some(handler)).await
}

#[cfg(unix)]
async fn run_with_control(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
    control: Option<AsyncControlHandler>,
) -> Result<ProcessResult, String> {
    use std::{
        os::unix::process::{CommandExt, ExitStatusExt},
        process::Stdio,
    };
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
    command.as_std_mut().process_group(0);
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            owner.phase = "not_spawned".into();
            record_owner(&spec.log_dir, &owner)?;
            return Err(format!("spawn agent: {error}"));
        }
    };
    let group = ProcessGroup(child.id().ok_or("missing child PID")? as i32);
    owner.pid = child.id();
    owner.process_group = Some(group.0);
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
    let (input_task, out_task) = if let Some(handler) = control {
        (
            None,
            tokio::spawn(async move {
                let result = control_io(
                    output,
                    input,
                    out_file,
                    spec.input,
                    handler,
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
            group.signal(libc::SIGKILL);
            child.wait().await.map_err(|e| e.to_string())?
        }
        None => {
            group.signal(libc::SIGINT);
            let status = match tokio::time::timeout(spec.signal_grace, child.wait()).await {
                Ok(status) => status,
                Err(_) => {
                    group.signal(libc::SIGTERM);
                    match tokio::time::timeout(spec.signal_grace, child.wait()).await {
                        Ok(status) => status,
                        Err(_) => {
                            group.signal(libc::SIGKILL);
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
    owner.termination_signal = status.signal();
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
        termination_signal: status.signal(),
        timed_out,
        stopped,
        stdout,
        stderr,
    })
}

#[cfg(not(unix))]
pub async fn run(_spec: ProcessSpec) -> Result<ProcessResult, String> {
    Err("process-tree containment is currently supported only on Unix".into())
}

#[cfg(not(unix))]
pub async fn run_until(
    _spec: ProcessSpec,
    _stop_file: Option<PathBuf>,
) -> Result<ProcessResult, String> {
    Err("process-tree containment is currently supported only on Unix".into())
}
