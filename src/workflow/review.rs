use super::*;

pub(super) fn review_binding(report: &Report) -> Result<String, String> {
    let drift = super::guardrail::charter_drift(&report.plan, report.charter.as_ref());
    if !drift.is_empty() {
        return Err(super::guardrail::refusal(&drift));
    }
    Ok(digest(
        &serde_json::to_vec(&(
            &report.state,
            &report.plan,
            &report.charter,
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
        && super::guardrail::charter_drift(&report.plan, report.charter.as_ref()).is_empty()
        && report.error.is_none()
        && report.candidate.is_some()
        && report.diff_hash.is_some()
        && report.tests.len() == report.plan.tasks.len()
        && report
            .tests
            .iter()
            .all(|t| t.exit_code == Some(0) && !t.timed_out)
        && report.tasks.iter().all(|t| t.status == "completed")
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

pub(super) fn apply_typed_review(
    dir: &Path,
    report: &mut Report,
    proposal: &crate::coordinator::ReviewProposal,
    store: &mut Store,
) -> Result<(), String> {
    let stopped = report.state == "stopped";
    match proposal.verdict {
        crate::coordinator::Verdict::Revise => {
            for revision in &proposal.revisions {
                let index = task_index(report, &revision.task)?;
                let old_test_command = revision
                    .test_command
                    .as_ref()
                    .map(|_| report.plan.tasks[index].test_command.clone());
                match super::amendments::apply(report, store, index, revision)? {
                    super::amendments::Outcome::Escalated(reason) => {
                        report.state = "blocked".into();
                        report.review_error = Some(reason);
                        store.record("approval", &serde_json::Value::Null, "Coordinator amendment needs human approval; plan change was not applied")?;
                        return Ok(());
                    }
                    super::amendments::Outcome::Applied => {}
                    super::amendments::Outcome::None => {}
                }
                let cause = report.tasks[index]
                    .error
                    .clone()
                    .unwrap_or_else(|| proposal.summary.clone());
                super::step_lessons::note(dir, store, &revision.task, &cause, &revision.feedback)?;
                // A test command that cannot work in this project is replaced, and the change
                // is recorded because the operator approved the plan with the old one.
                if let (Some(command), Some(old)) = (&revision.test_command, old_test_command) {
                    let response = format!(
                        "Replace test command {old:?} with {command:?}: {}",
                        revision.feedback
                    );
                    super::step_lessons::note(dir, store, &revision.task, &cause, &response)?;
                    store.record(
                        "test_command_revised",
                        &serde_json::json!({"task":revision.task,"from":old,"to":command,"reason":revision.feedback}),
                        "Coordinator replaced a test command that could not work in this project",
                    )?;
                }
                report.tasks[index].feedback = Some(revision.feedback.clone());
                report.tasks[index].restriction = None;
                invalidate(report, index, store)?;
            }
        }
        crate::coordinator::Verdict::Blocked => {
            report.state = "blocked".into();
            store.record(
                "approval",
                &serde_json::Value::Null,
                "Blocked quality review requires human intervention",
            )?;
        }
        crate::coordinator::Verdict::Accept => {}
    }
    if stopped {
        report.state = "stopped".into();
    }
    Ok(())
}

pub fn apply_review(dir: &Path, expected: &str) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    editable(&report)?;
    let _repository_lease = crate::leases::WorkspaceLease::acquire(&report.repo, dir)?;
    validate_checkpoints(&report, dir)?;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    let receipt = store.document("review_receipt")?;
    if receipt["proposal_sha256"] != expected || receipt["binding"] != review_binding(&report)? {
        return Err("review proposal or frozen inputs changed; application refused".into());
    }
    if store
        .document("review_applied")
        .is_ok_and(|v| v["round"] == receipt["round"] && v["proposal_sha256"] == expected)
    {
        return Err("review proposal was already applied".into());
    }
    for source in receipt["sources"]
        .as_array()
        .ok_or("missing review evidence sources")?
    {
        let path = source["path"]
            .as_str()
            .ok_or("invalid review evidence path")?;
        if source["sha256"] != file_digest(Path::new(path))? {
            return Err("review evidence changed; application refused".into());
        }
    }
    let proposal: crate::coordinator::ReviewProposal =
        serde_json::from_value(receipt["proposal"].clone()).map_err(|e| e.to_string())?;
    if digest(&serde_json::to_vec(&proposal).map_err(|e| e.to_string())?) != expected {
        return Err("saved review proposal content changed".into());
    }
    proposal.validate(
        &report
            .tasks
            .iter()
            .map(|t| t.id.clone())
            .collect::<Vec<_>>(),
        &known_review_evidence(&report),
        candidate_verified(&report),
    )?;
    apply_typed_review(dir, &mut report, &proposal, &mut store)?;
    store.record(
        "review_applied",
        &serde_json::json!({"round":receipt["round"],"proposal_sha256":expected}),
        "Human applied the exact saved scoped review; no additional model call or merge approval",
    )?;
    save(&mut store, dir, &report)
}

pub(super) fn evidence_excerpt(path: &Path, limit: usize) -> Result<serde_json::Value, String> {
    let mut bytes = vec![];
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let truncated = bytes.len() > limit;
    bytes.truncate(limit);
    Ok(
        serde_json::json!({"path":path,"sha256":file_digest(path)?,"text":String::from_utf8_lossy(&bytes),"truncated":truncated}),
    )
}

mod orchestration;
pub use orchestration::review;
pub(super) use orchestration::review_as;
pub(super) use orchestration::review_request;
