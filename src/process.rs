use std::{path::PathBuf, time::Duration};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Ownership {
    pub coordinator_pid: u32,
    pub pid: Option<u32>,
    pub process_group: Option<i32>,
    pub phase: String,
    pub exit_code: Option<i32>,
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

fn record_owner(dir: &std::path::Path, owner: &Ownership) -> Result<(), String> {
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
    pub timed_out: bool,
    pub stopped: bool,
    pub stdout: PathBuf,
    pub stderr: PathBuf,
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
    use std::{os::unix::process::CommandExt, process::Stdio};
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
            timed_out: false,
            stopped: true,
            stdout,
            stderr,
        });
    }
    let mut command = Command::new(&spec.executable);
    command
        .args(&spec.args)
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
    let input_task = tokio::spawn(async move {
        input.write_all(&spec.input).await?;
        input.shutdown().await
    });
    let out_task = tokio::spawn(async move {
        let mut file = out_file;
        tokio::io::copy(&mut output, &mut file).await?;
        file.flush().await
    });
    let err_task = tokio::spawn(async move {
        let mut file = err_file;
        tokio::io::copy(&mut errors, &mut file).await?;
        file.flush().await
    });

    let stop_signal = async {
        match stop_file {
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
    let status = tokio::select! {
        biased;
        status=child.wait()=>Some(status.map_err(|e|e.to_string())?),
        signal=stop_signal=>{signal.map_err(|e|e.to_string())?;stopped=true;None},
        _=tokio::time::sleep(spec.timeout)=>{timed_out=true;None},
    };
    let status = match status {
        Some(status) => status,
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
    owner.phase = "terminal".into();
    owner.exit_code = status.code();
    owner.stopped = stopped;
    owner.timed_out = timed_out;
    record_owner(&spec.log_dir, &owner)?;
    let input_result = input_task.await.map_err(|e| e.to_string())?;
    out_task
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    err_task
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    if status.success() && !timed_out && !stopped {
        input_result.map_err(|e| format!("write agent input: {e}"))?;
    }
    Ok(ProcessResult {
        exit_code: status.code(),
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
