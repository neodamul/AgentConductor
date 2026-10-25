use agentconductor::workflow;
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub fn retry_schedule(
    mut attempt: impl FnMut(usize) -> Result<bool, String>,
) -> Result<bool, String> {
    for number in 1..=3 {
        if attempt(number)? {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn wait_for_manual_schedule(
    run: &Path,
    expected: i64,
    timeout: Duration,
) -> Result<bool, String> {
    let deadline = Instant::now() + timeout;
    loop {
        let report = workflow::load(run).map_err(|error| error.to_string())?;
        if report
            .tasks
            .first()
            .and_then(|task| task.restriction.as_ref())
            .is_some_and(|restriction| {
                restriction.source == "manual_schedule"
                    && restriction.available_at == Some(expected)
            })
        {
            return Ok(true);
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub fn diagnostics(run: &Path, driver_output: &str) -> String {
    let report = workflow::load(run)
        .map(|report| {
            let task = report.tasks.first().map(|task| {
                format!(
                    "{}:{:?}",
                    task.status,
                    task.restriction
                        .as_ref()
                        .map(|restriction| (&restriction.source, restriction.available_at))
                )
            });
            format!("state={}, task={task:?}", report.state)
        })
        .unwrap_or_else(|error| format!("report_error={error}"));
    let actions = std::fs::read_dir(run)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("coordinator-ui-")
                .then(|| {
                    let process = entry.path().join("process");
                    format!(
                        "owner={}, stderr={}",
                        std::fs::read_to_string(process.join("owner.json")).unwrap_or_default(),
                        std::fs::read_to_string(process.join("stderr.log")).unwrap_or_default()
                    )
                })
        })
        .collect::<Vec<_>>()
        .join(" | ");
    format!("ui={driver_output}; report={report}; actions={actions}")
}
