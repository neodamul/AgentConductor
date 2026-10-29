//! GitHub #234: which steps of a request are worth a line in the request diagnostics log.
use std::time::Duration;

/// A step shorter than this is not logged.
pub const THRESHOLD: Duration = Duration::from_millis(500);

/// The log line for a step (`lock` is the wait for the settings lock), or `None` when it was quick.
pub fn line(name: &str, elapsed: Duration) -> Option<String> {
    (elapsed >= THRESHOLD).then(|| format!("step name={name} ms={}", elapsed.as_millis()))
}
