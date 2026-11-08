//! GitHub #568: a `/usage` screen that stays unreadable and silent is not waited for until the limit
//! (a signed-out CLI or one billed per API use shows no subscription windows); the screen is named at once.
use std::time::Duration;

const CAP_SECS: u64 = 20;

/// How long the screen must stay silent before an unreadable answer is final.
pub(super) fn after(wait: Duration) -> Duration {
    (wait / 4).min(Duration::from_secs(CAP_SECS))
}

/// Ends the wait with the unreadable reason once the screen has been silent for `limit`.
pub(super) fn check(
    sent: bool,
    quiet: u64,
    limit: Duration,
    reason: Option<&str>,
) -> Result<(), String> {
    match reason.filter(|_| sent && Duration::from_secs(quiet) >= limit) {
        Some(reason) => Err(format!(
            "Claude quota screen not understood after /usage: {reason}; if this Claude CLI is signed out or billed per API use, run claude, sign in with /login to your subscription, then refresh; no generation requested"
        )),
        None => Ok(()),
    }
}
