use super::*;

pub(super) fn binding(dir: &Path, report: &Report) -> Result<String, String> {
    let git = Repository::open(&report.repo)?;
    if git.head()? != report.base || git.branch()? != report.branch {
        return Err("target branch changed since verification".into());
    }
    let candidate = report.candidate.as_ref().ok_or("no verified candidate")?;
    if report.plan.workspace_mode == WorkspaceMode::Shared
        && !git
            .command(&["diff", "--cached", "--name-only"])?
            .is_empty()
    {
        return Err("shared Git index changed since verification".into());
    }
    if digest(&git.diff(&report.base, &candidate.commit)?)
        != report.diff_hash.clone().ok_or("missing diff hash")?
    {
        return Err("candidate Git diff differs from verified evidence".into());
    }
    if git.text(&["rev-parse", &format!("{}^{{tree}}", candidate.commit)])? != candidate.tree {
        return Err("candidate tree identity mismatch".into());
    }
    if file_digest(&dir.join("diff.patch"))?
        != report.diff_hash.clone().ok_or("diff evidence missing")?
    {
        return Err("diff evidence changed".into());
    }
    for test in &report.tests {
        if test.exit_code != Some(0)
            || test.timed_out
            || file_digest(&test.stdout)? != test.stdout_hash
            || file_digest(&test.stderr)? != test.stderr_hash
        {
            return Err("test evidence changed or failed".into());
        }
    }
    if report.tests.len() != report.plan.tasks.len() {
        return Err("integration tests incomplete".into());
    }
    let verification = Repository::open(&report.verification_cwd)?;
    if report.plan.workspace_mode == WorkspaceMode::Worktree {
        git.clean()?;
    }
    let base = if report.plan.workspace_mode == WorkspaceMode::Worktree {
        &candidate.commit
    } else {
        &report.base
    };
    if verification
        .snapshot(base, &scopes(&report.plan), &dir.join("approval.index"))?
        .tree
        != candidate.tree
    {
        return Err("candidate files changed since verification".into());
    }
    let value = serde_json::json!({"repo":report.repo,"base":report.base,"branch":report.branch,"candidate":candidate,"plan":report.plan,"diff_hash":report.diff_hash,"tests":report.tests});
    Ok(digest(
        &serde_json::to_vec(&value).map_err(|e| e.to_string())?,
    ))
}

pub fn approve(dir: &Path, candidate: &str) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    if report.state != "waiting_approval" {
        return Err("run is not waiting for approval".into());
    }
    if report.candidate.as_ref().map(|c| c.commit.as_str()) != Some(candidate) {
        return Err("approval requires the exact candidate SHA".into());
    }
    let fingerprint = binding(dir, &report)?;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record(
        "approval",
        &serde_json::json!({"fingerprint":fingerprint,"candidate":candidate}),
        "Human approved exact integration candidate",
    )?;
    report.state = "approved".into();
    save(&mut store, dir, &report)
}
pub fn reject(dir: &Path, reason: &str) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    if !matches!(report.state.as_str(), "waiting_approval" | "approved") {
        return Err("run has no pending approval".into());
    }
    report.state = "rejected".into();
    report.error = Some(reason.into());
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record(
        "approval",
        &serde_json::Value::Null,
        "Human rejected candidate",
    )?;
    save(&mut store, dir, &report)
}
pub fn merge(dir: &Path) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    let _repository_lease = crate::leases::WorkspaceLease::acquire(&report.repo, dir)?;
    if report.state != "approved" {
        return Err("human approval is required before merge".into());
    }
    let mut store = Store::open(dir.join("state.sqlite"))?;
    let approval = store.document("approval")?;
    let fingerprint = binding(dir, &report)?;
    if approval["fingerprint"] != fingerprint {
        return Err("approval no longer matches candidate and evidence".into());
    }
    let git = Repository::open(&report.repo)?;
    let candidate = &report.candidate.as_ref().unwrap().commit;
    match report.plan.workspace_mode {
        WorkspaceMode::Shared => git.merge_shared(&report.branch, &report.base, candidate)?,
        WorkspaceMode::Worktree => {
            git.command(&["merge", "--ff-only", candidate])?;
        }
    }
    report.state = "completed".into();
    save(&mut store, dir, &report)?;
    // DD-62: the merge is final; a failed tree update is reported, not rolled back.
    sync_phase(dir).map_err(|e| format!("merged, but the slug tree update failed: {e}"))
}
