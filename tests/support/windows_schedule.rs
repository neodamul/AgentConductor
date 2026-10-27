use agentconductor::workflow;
use std::{
    path::Path,
    time::{Duration, Instant},
};

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
