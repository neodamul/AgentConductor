use crate::{
    plan::AgentProfile,
    process::{self, ProcessResult, ProcessSpec},
};
use std::{path::Path, time::Duration};

pub async fn run(
    profile: &AgentProfile,
    args: Vec<String>,
    prompt: &str,
    input_mode: crate::adapters::InputMode,
    snapshot: (&crate::git::Repository, &str, &Path),
    log_dir: &Path,
    stop_file: &Path,
) -> Result<ProcessResult, String> {
    snapshot.0.export_tree(snapshot.1, snapshot.2)?;
    let process = process::run_until(
        ProcessSpec {
            executable: profile.executable.clone().into(),
            args,
            cwd: snapshot.2.to_path_buf(),
            input: if input_mode == crate::adapters::InputMode::Stdin {
                prompt.as_bytes().to_vec()
            } else {
                vec![]
            },
            log_dir: log_dir.to_path_buf(),
            timeout: Duration::from_secs(1800),
            signal_grace: Duration::from_secs(5),
            environment: profile.environment.clone(),
        },
        Some(stop_file.to_path_buf()),
    )
    .await;
    let removed = std::fs::remove_dir_all(snapshot.2);
    let process = process?;
    removed.map_err(|e| format!("planning snapshot cleanup failed: {e}"))?;
    if process.exit_code != Some(0) || process.timed_out || process.stopped {
        return Err(crate::coordinator::failure(
            "planning coordinator",
            &process,
        ));
    }
    Ok(process)
}
