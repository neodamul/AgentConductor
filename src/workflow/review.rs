mod apply;
mod request;

pub use apply::apply_review;
pub(super) use apply::apply_typed_review;
pub use request::review;
pub(super) use request::{review_as, review_request};

use super::*;

pub(super) fn review_binding(report: &Report) -> Result<String, String> {
    Ok(digest(
        &serde_json::to_vec(&(
            &report.state,
            &report.plan,
            &report.base,
            &report.branch,
            &report.tasks,
            &report.candidate,
            &report.tests,
            &report.diff_hash,
            &report.checkpoints,
        ))
        .map_err(|e| e.to_string())?,
    ))
}

pub(super) fn candidate_verified(report: &Report) -> bool {
    matches!(report.state.as_str(), "waiting_approval" | "approved")
        && report.error.is_none()
        && report.candidate.is_some()
        && report.diff_hash.is_some()
        && report.tests.len() == report.plan.tasks.len()
        && report.tests.iter().all(|t| report.test_passed(t))
        && report.tasks.iter().all(|t| t.status == "completed")
}

pub(super) const PENDING_RED_REVIEW: &str =
    "integration verification requires coordinator review for zero-exit Red tests";

fn pending_red_review(report: &Report) -> bool {
    report.state == "failed" && report.error.as_deref() == Some(PENDING_RED_REVIEW)
}

fn validate_red_acceptance(
    report: &Report,
    proposal: &crate::coordinator::ReviewProposal,
) -> Result<(), String> {
    if pending_red_review(report) && proposal.verdict == crate::coordinator::Verdict::Accept {
        for test in report
            .tests
            .iter()
            .filter(|test| report.zero_exit_red(test))
        {
            if !proposal.evidence.contains(&format!("test:{}", test.task)) {
                return Err(format!(
                    "acceptance must cite test evidence for {}",
                    test.task
                ));
            }
            if fs::metadata(&test.stdout).map_err(|e| e.to_string())?.len() == 0
                && fs::metadata(&test.stderr).map_err(|e| e.to_string())?.len() == 0
            {
                return Err(format!(
                    "zero-exit Red test produced no evidence for {}",
                    test.task
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn known_review_evidence(report: &Report) -> Vec<String> {
    let mut ids = report
        .tasks
        .iter()
        .map(|t| format!("agent:{}", t.id))
        .collect::<Vec<_>>();
    ids.extend(report.tests.iter().map(|t| format!("test:{}", t.task)));
    if report.diff_hash.is_some() {
        ids.push("diff:combined".into());
    }
    ids
}

pub(super) fn collect_review_sources(
    value: &serde_json::Value,
    sources: &mut Vec<serde_json::Value>,
) {
    if let (Some(path), Some(hash)) = (
        value.get("path").and_then(serde_json::Value::as_str),
        value.get("sha256").and_then(serde_json::Value::as_str),
    ) {
        sources.push(serde_json::json!({"path":path,"sha256":hash}));
    }
    match value {
        serde_json::Value::Object(values) => {
            for item in values.values() {
                collect_review_sources(item, sources);
            }
        }
        serde_json::Value::Array(values) => {
            for item in values {
                collect_review_sources(item, sources);
            }
        }
        _ => {}
    }
}

pub(super) fn evidence_excerpt(path: &Path, limit: usize) -> Result<serde_json::Value, String> {
    log_excerpt(path, limit, false)
}

fn log_excerpt(path: &Path, limit: usize, include_tail: bool) -> Result<serde_json::Value, String> {
    use std::io::{Seek, SeekFrom};
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut bytes = vec![];
    (&mut file)
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let truncated = bytes.len() > limit;
    bytes.truncate(limit);
    if truncated && include_tail {
        let tail = limit / 2;
        bytes.truncate(limit - tail);
        bytes.extend_from_slice(b"\n[... omitted ...]\n");
        file.seek(SeekFrom::End(-(tail as i64)))
            .map_err(|e| e.to_string())?;
        file.take(tail as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
    }
    Ok(
        serde_json::json!({"path":path,"sha256":file_digest(path)?,"text":String::from_utf8_lossy(&bytes),"truncated":truncated}),
    )
}
