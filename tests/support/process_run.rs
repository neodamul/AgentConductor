use agentconductor::process::{self, ProcessResult, ProcessSpec};
use std::{path::Path, time::Duration};

pub async fn until_marker(
    spec: ProcessSpec,
    marker: &Path,
    readiness_timeout: Duration,
) -> Result<(ProcessResult, bool), String> {
    let process = tokio::spawn(process::run(spec));
    let deadline = tokio::time::Instant::now() + readiness_timeout;
    loop {
        let ready = marker.exists();
        if ready || process.is_finished() || tokio::time::Instant::now() >= deadline {
            let result = process.await.map_err(|error| error.to_string())??;
            return Ok((result, ready));
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
