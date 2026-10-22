//! DD-184: coordinator-directed feedback reuses evidence review and its validated amendments.
use super::*;
use serde_json::{Value, json};

const MAX_BYTES: usize = 8 * 1024;

#[cfg(test)]
#[path = "../../tests/support/coordinator_request_retry.rs"]
mod retry_tests;

async fn retry_review<T, F, Fut>(deadline: std::time::Instant, mut review: F) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    loop {
        if std::time::Instant::now() >= deadline {
            return Err("coordinator request timed out while waiting for the active run".into());
        }
        match review().await {
            Err(error)
                if error.contains("lease is held") && std::time::Instant::now() < deadline =>
            {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            result => return result,
        }
    }
}

fn note(dir: &Path, id: &str, status: &str, message: &str, extra: Value) -> Result<(), String> {
    let mut payload = json!({"id":id,"status":status,"message":message});
    if let (Some(map), Some(more)) = (payload.as_object_mut(), extra.as_object()) {
        map.extend(more.clone());
    }
    Store::open(dir.join("state.sqlite"))?.record(
        "coordinator_request",
        &payload,
        &format!("Coordinator request {id} is {status}"),
    )
}

/// Sends a bounded operator request to the recorded coordinator and applies only validated revisions.
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
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout.clamp(1, 1800));
    let mut available = false;
    while std::time::Instant::now() < deadline {
        match crate::leases::Lease::acquire(&dir.join("coordinator.lock")) {
            Ok(lock) => {
                drop(lock);
                available = true;
                break;
            }
            Err(error) if error.contains("lease is held") => {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(error) => {
                note(dir, &id, "failed", message, json!({"error":error}))?;
                return Err(error);
            }
        }
    }
    if !available {
        let error = "coordinator request timed out while waiting for the active run";
        note(dir, &id, "failed", message, json!({"error":error}))?;
        return Err(error.into());
    }
    note(dir, &id, "processing", message, Value::Null)?;
    let outcome = retry_review(deadline, || {
        super::review::review_request(dir, profile.clone(), message)
    })
    .await;
    match outcome {
        Ok(proposal) => {
            if proposal.verdict == crate::coordinator::Verdict::Revise
                && load(dir)?.state != "blocked"
            {
                let current = (now() / 1000) as i64;
                let result = async {
                    let report = resume_due(dir, current, timeout).await?;
                    repair(dir, timeout, report).await.map(|_| ())
                }
                .await;
                if let Err(error) = result {
                    note(dir, &id, "failed", message, json!({"error":error}))?;
                    return Err(error);
                }
            }
            note(
                dir,
                &id,
                "completed",
                message,
                json!({"verdict":proposal.verdict,"summary":proposal.summary}),
            )?;
            Ok(proposal)
        }
        Err(error) => {
            note(dir, &id, "failed", message, json!({"error":error}))?;
            Err(error)
        }
    }
}
