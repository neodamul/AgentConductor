use super::*;
mod lock;
mod worker;
pub(crate) use lock::{Lock, validate_permissions};
pub(super) use worker::worker;

pub async fn run(
    plan: Plan,
    repo_path: &Path,
    run_dir: &Path,
    timeout: u64,
) -> Result<Report, String> {
    run_inner(plan, repo_path, run_dir, timeout, None).await
}

pub async fn run_reviewed(
    plan: Plan,
    repo_path: &Path,
    run_dir: &Path,
    timeout: u64,
    expected_base: &str,
) -> Result<Report, String> {
    run_inner(plan, repo_path, run_dir, timeout, Some(expected_base)).await
}

pub(super) async fn run_inner(
    plan: Plan,
    repo_path: &Path,
    run_dir: &Path,
    timeout: u64,
    expected_base: Option<&str>,
) -> Result<Report, String> {
    plan.validate()?;
    validate_permissions(&plan)?;
    if timeout == 0 {
        return Err("timeout must be positive".into());
    }
    let parent = run_dir
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let git = crate::folder::working_repository(
        repo_path,
        &parent.join("folder-history"),
        expected_base,
    )?;
    let mut repository_lease = crate::leases::WorkspaceLease::acquire(&git.root, run_dir)?;
    git.clean()?;
    let base = git.head()?;
    let folder = crate::folder::load(&git.root)?;
    if let Some(source) = &folder {
        source.validate()?;
        if source.base != base {
            return Err("folder baseline changed before execution".into());
        }
    }
    if expected_base.is_some_and(|reviewed| reviewed != base) {
        return Err("repository base changed since plan approval".into());
    }
    let branch = git.branch()?;
    fs::create_dir(run_dir).map_err(|e| format!("create new run directory: {e}"))?;
    let dir = run_dir.canonicalize().map_err(|e| e.to_string())?;
    if dir.starts_with(&git.root) {
        return Err("run directory must be outside the repository".into());
    }
    let _lock = Lock::acquire(&dir)?;
    let worktree = plan.workspace_mode == WorkspaceMode::Worktree;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record(
        "execution_baseline",
        &serde_json::json!({"repo":git.root,"commit":base,"external_history":git.is_external()}),
        "Recorded immutable baseline before launching any task",
    )?;
    let mut report = Report {
        folder,
        account_restrictions: Default::default(),
        review_round: 0,
        latest_review: None,
        review_error: None,
        schema_version: 1,
        state: "running".into(),
        repo: git.root.clone(),
        base: base.clone(),
        branch,
        plan: plan.clone(),
        tasks: vec![],
        candidate: None,
        tests: vec![],
        diff_hash: None,
        error: None,
        verification_cwd: git.root.clone(),
        verification_attempt: 0,
        checkpoints: vec![],
    };
    for (i, task) in plan.tasks.iter().enumerate() {
        let artifacts = dir.join(format!("task-{i}"));
        fs::create_dir(&artifacts).map_err(|e| e.to_string())?;
        let cwd = if worktree {
            let cwd = dir.join(format!("workspace-{i}"));
            git.worktree(&cwd, &base)?;
            cwd
        } else {
            git.root.clone()
        };
        report.tasks.push(TaskReport {
            approval_epoch: 0,
            pending_tool: None,
            tool_request: None,
            session_established: false,
            feedback: None,
            checkpoint_context: None,
            dependencies: vec![],
            attempt: 0,
            restriction: None,
            retry_not_before: 0,
            automatic_retries: 0,
            timeouts: 0,
            id: task.id.clone(),
            status: "pending".into(),
            cwd,
            started_ms: None,
            finished_ms: None,
            session_id: None,
            error: None,
            base: base.clone(),
            snapshot: None,
            artifacts,
        });
    }
    save(&mut store, &dir, &report)?;
    drive(report, &dir, &mut store, timeout, &mut repository_lease).await
}

pub(super) fn prepare_dependencies(
    report: &mut Report,
    index: usize,
    dir: &Path,
    store: &mut Store,
    context: &[DependencyContext],
    snapshots: &[Snapshot],
) -> Result<(), String> {
    let task = &report.tasks[index];
    if task.attempt == 0 {
        let workspace = Repository::open(&task.cwd)?;
        for snapshot in snapshots {
            workspace.command(&["merge", "--no-edit", &snapshot.commit])?;
        }
        report.tasks[index].base = workspace.head()?;
        return Ok(());
    }
    if task.dependencies == context {
        return Ok(());
    }
    let previous_cwd = task.cwd.clone();
    let previous_session = task.session_id.clone();
    let previous_base = task.base.clone();
    let previous = Repository::open(&previous_cwd)?;
    if previous.head()? != previous_base {
        return Err("task changed its Git baseline before dependency refresh".into());
    }
    let partial = previous.snapshot(
        &previous_base,
        &report.plan.tasks[index].paths,
        &dir.join(format!("partial-{index}-{}.index", task.attempt)),
    )?;
    let patch = previous.diff(&previous_base, &partial.commit)?;
    let workspace_path = dir.join(format!("workspace-{index}-attempt-{}", task.attempt + 1));
    let evidence = serde_json::json!({"task":task.id,"from":previous_cwd,"to":workspace_path,"previous_session":previous_session,"partial":partial,"dependencies":context});
    fs::write(
        task.artifacts.join("dependency-refresh.json"),
        serde_json::to_vec_pretty(&evidence).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let git = Repository::open(&report.repo)?;
    git.command(&[
        "update-ref",
        &format!(
            "refs/codex/agentconductor/{}/{}",
            digest(dir.to_string_lossy().as_bytes()),
            partial.commit
        ),
        &partial.commit,
    ])?;
    store.record(
        "dependency_refresh",
        &evidence,
        "Retained prior checkout and scoped partial changes; prepare a fresh dependency workspace",
    )?;
    git.worktree(&workspace_path, &report.base)?;
    let workspace = Repository::open(&workspace_path)?;
    for snapshot in snapshots {
        workspace.command(&["merge", "--no-edit", &snapshot.commit])?;
    }
    let base = workspace.head()?;
    workspace.apply_working_patch(&patch)?;
    workspace.check_scope(&workspace.changed(&base)?, &report.plan.tasks[index].paths)?;
    report.tasks[index].cwd = workspace.root;
    report.tasks[index].base = base;
    report.tasks[index].session_id = None;
    Ok(())
}
