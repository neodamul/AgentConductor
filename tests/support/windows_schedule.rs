use agentconductor::workflow;
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

pub fn wait_for_ui_action_dispatch(run: &Path, timeout: Duration) -> Result<bool, String> {
    let deadline = Instant::now() + timeout;
    loop {
        let dispatched = fs::read_dir(run)
            .map_err(|error| error.to_string())?
            .filter_map(Result::ok)
            .any(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("coordinator-ui-")
                    && entry.path().join("process/owner.json").is_file()
            });
        if dispatched {
            return Ok(true);
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
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
