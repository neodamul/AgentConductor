use super::*;
/// Sleeps for `timeout` of unpaused time.
async fn unpaused(
    timeout: Duration,
    paused: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
) {
    let Some(paused) = paused else {
        return tokio::time::sleep(timeout).await;
    };
    let mut left = timeout;
    while !left.is_zero() {
        let step = left.min(Duration::from_millis(250));
        tokio::time::sleep(step).await;
        if !paused.load(std::sync::atomic::Ordering::Acquire) {
            left = left.saturating_sub(step);
        }
    }
}

pub(super) async fn run_with_control(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
    control: Option<(AsyncControlHandler, Duration)>,
    paused: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
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
    // Windows only appends .exe: resolve npm .cmd shims and bare names like `claude`.
    let executable = crate::executable::resolve(&spec.executable.to_string_lossy())
        .unwrap_or_else(|| spec.executable.clone());
    let mut command = Command::new(executable);
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
        _=unpaused(spec.timeout,paused)=>{timed_out=true;None},
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
