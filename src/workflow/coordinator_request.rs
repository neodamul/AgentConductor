//! DD-182: feedback the operator sends to the coordinator. It runs as a review round of the
//! run's recorded coordinator with the operator's text added, and each state is an audit record.
use super::*;
use serde_json::{Value, json};

const MAX_BYTES: usize = 8 * 1024;
/// How long a request waits for the run lock before it fails (500 ms steps).
const LOCK_WAIT_STEPS: u32 = 240;

fn note(dir: &Path, id: &str, status: &str, message: &str, extra: Value) -> Result<(), String> {
    let mut payload =
        json!({"id":id,"status":status,"message":message.chars().take(2000).collect::<String>()});
    if let (Some(map), Some(more)) = (payload.as_object_mut(), extra.as_object()) {
        map.extend(more.clone());
    }
    Store::open(dir.join("state.sqlite"))?.record(
        "coordinator_request",
        &payload,
        &format!("Coordinator request {id} is {status}"),
    )
}

/// Sends `message` to the run's coordinator and acts on its typed answer.
pub async fn coordinator_request(
    dir: &Path,
    timeout: u64,
    message: &str,
) -> Result<crate::coordinator::ReviewProposal, String> {
    let message = message.trim();
    if message.is_empty() || message.len() > MAX_BYTES || message.contains('\0') {
        return Err("a coordinator request needs 1 byte to 8 KiB of text without NUL".into());
    }
    let profile = crate::native_guard::orchestrator_profile(dir)
        .ok_or("no coordinator is recorded for this run; register one before sending a request")?;
    let id = format!(
        "rq-{}",
        &digest(format!("{message}{}", now()).as_bytes())[..10]
    );
    note(dir, &id, "queued", message, Value::Null)?;
    // Waiting for the run lock is the queued state: another step holds the run.
    for _ in 0..LOCK_WAIT_STEPS {
        match Lock::acquire(dir) {
            Ok(lock) => {
                drop(lock);
                break;
            }
            Err(e) if e.contains("lease is held") => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            Err(e) => {
                note(dir, &id, "failed", message, json!({"error":e}))?;
                return Err(e);
            }
        }
    }
    note(dir, &id, "processing", message, Value::Null)?;
    // The room polls the run and may hold the lock between the wait above and the review.
    let mut retries = 0;
    let outcome = loop {
        match super::review::review_request(dir, profile.clone(), message).await {
            Err(e) if e.contains("lease is held") && retries < LOCK_WAIT_STEPS => {
                retries += 1;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            other => break other,
        }
    };
    match outcome {
        Ok(proposal) => {
            note(
                dir,
                &id,
                "completed",
                message,
                json!({"verdict":proposal.verdict,"summary":proposal.summary}),
            )?;
            if proposal.verdict == crate::coordinator::Verdict::Revise
                && load(dir)?.state != "blocked"
            {
                let current = (now() / 1000) as i64;
                let report = super::resume_due(dir, current, timeout).await?;
                super::repair(dir, timeout, report).await?;
            }
            Ok(proposal)
        }
        Err(error) => {
            note(dir, &id, "failed", message, json!({"error":error}))?;
            Err(error)
        }
    }
}
