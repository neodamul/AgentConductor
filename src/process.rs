use std::{path::PathBuf, time::Duration};

pub struct ProcessSpec {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub input: Vec<u8>,
    pub log_dir: PathBuf,
    pub timeout: Duration,
    pub signal_grace: Duration,
}

pub struct ProcessResult {
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub stdout: PathBuf,
    pub stderr: PathBuf,
}

#[cfg(unix)]
struct ProcessGroup(i32);

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
    let mut command = Command::new(&spec.executable);
    command
        .args(&spec.args)
        .current_dir(&spec.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    command.as_std_mut().process_group(0);
    let mut child = command.spawn().map_err(|e| format!("spawn agent: {e}"))?;
    let group = ProcessGroup(child.id().ok_or("missing child PID")? as i32);
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

    let (status, timed_out) = match tokio::time::timeout(spec.timeout, child.wait()).await {
        Ok(status) => (status.map_err(|e| e.to_string())?, false),
        Err(_) => {
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
            (status.map_err(|e| e.to_string())?, true)
        }
    };
    // Descendants must not outlive this attempt or keep its pipes open.
    drop(group);
    let input_result = input_task.await.map_err(|e| e.to_string())?;
    out_task
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    err_task
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    if status.success() && !timed_out {
        input_result.map_err(|e| format!("write agent input: {e}"))?;
    }
    Ok(ProcessResult {
        exit_code: status.code(),
        timed_out,
        stdout,
        stderr,
    })
}

#[cfg(not(unix))]
pub async fn run(_spec: ProcessSpec) -> Result<ProcessResult, String> {
    Err("process-tree containment is currently supported only on Unix".into())
}
