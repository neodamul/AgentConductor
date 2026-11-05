use super::*;

pub(in crate::workflow) fn apply_typed_review(
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
                // DD-174: what the coordinator learned reaches the operator and later tasks.
                let cause = report.tasks[index]
                    .error
                    .clone()
                    .unwrap_or_else(|| proposal.summary.clone());
                let dir_lesson = |store: &mut Store, response: &str| {
                    super::step_lessons::note(dir, store, &revision.task, &cause, response)
                };
                dir_lesson(store, &revision.feedback)?;
                // DD-179: a typed amendment is applied when low risk; otherwise a person decides.
                if let super::amendments::Outcome::Escalated(reason) =
                    super::amendments::apply(report, store, index, revision)?
                {
                    report.state = "blocked".into();
                    report.review_error =
                        Some(format!("coordinator amendment needs a person: {reason}"));
                    store.record(
                        "approval",
                        &serde_json::Value::Null,
                        "Coordinator amendment needs a person; run stopped",
                    )?;
                    return Ok(());
                }
                // A test command that cannot work in this project is replaced, and the change
                // is recorded because the operator approved the plan with the old one.
                if let Some(command) = &revision.test_command {
                    let old = std::mem::replace(
                        &mut report.plan.tasks[index].test_command,
                        command.clone(),
                    );
                    let response = format!(
                        "Replace test command {old:?} with {command:?}: {}",
                        revision.feedback
                    );
                    dir_lesson(store, &response)?;
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
