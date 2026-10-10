use super::*;

pub(super) async fn drive(
    mut report: Report,
    dir: &Path,
    store: &mut Store,
    timeout: u64,
    repository_lease: &mut crate::leases::WorkspaceLease,
) -> Result<Report, String> {
    let git = Repository::open(&report.repo)?;
    let base = report.base.clone();
    let worktree = report.plan.workspace_mode == WorkspaceMode::Worktree;
    report.state = "running".into();
    save(store, dir, &report)?;
    repository_lease.claim()?;
    let mut jobs = tokio::task::JoinSet::new();
    loop {
        let plan = report.plan.clone();
        let ledger = crate::gates::Ledger::open(&dir.join("state.sqlite"))?;
        let mut exclusive = None;
        for (index, task) in report.tasks.iter().enumerate() {
            let ready = task.status == "running"
                || (task.status == "pending"
                    && plan.tasks[index].depends_on.iter().all(|id| {
                        report
                            .tasks
                            .iter()
                            .any(|t| &t.id == id && t.status == "completed")
                    }));
            if ready
                && let Some(id) = &task.tool_request
                && ledger.receipt(id)?.is_some_and(|r| r.status == "approved")
            {
                exclusive = Some(index);
                break;
            }
        }
        if dir.join("stop.request").exists() {
            for task in &mut report.tasks {
                if task.status == "pending" {
                    task.status = "stopped".into();
                }
            }
        }
        for i in 0..plan.tasks.len() {
            if exclusive.is_some_and(|only| only != i) {
                continue;
            }
            // Drain existing parallel work before consuming a workspace-bound permission.
            if exclusive == Some(i) && !jobs.is_empty() && report.tasks[i].status == "pending" {
                continue;
            }
            if report.tasks[i].status != "pending" {
                continue;
            }
            let deps: Vec<_> = plan.tasks[i]
                .depends_on
                .iter()
                .map(|id| report.tasks.iter().find(|t| &t.id == id).unwrap())
                .collect();
            if deps
                .iter()
                .any(|t| matches!(t.status.as_str(), "failed" | "blocked"))
            {
                report.tasks[i].status = "blocked".into();
                continue;
            }
            if deps.iter().any(|t| t.status != "completed") {
                continue;
            }
            let dependency_context: Vec<_> = deps
                .iter()
                .map(|dep| DependencyContext {
                    task: dep.id.clone(),
                    attempt: dep.attempt,
                    snapshot: dep.snapshot.as_ref().map(|s| s.commit.clone()),
                    paths: plan
                        .tasks
                        .iter()
                        .find(|t| t.id == dep.id)
                        .unwrap()
                        .paths
                        .clone(),
                    artifacts: dep.artifacts.clone(),
                })
                .collect();
            let dependency_snapshots: Vec<_> =
                deps.iter().filter_map(|dep| dep.snapshot.clone()).collect();
            let profile = plan
                .agents
                .iter()
                .find(|a| a.id == plan.tasks[i].agent)
                .unwrap();
            let active = report
                .tasks
                .iter()
                .enumerate()
                .filter(|(j, t)| t.status == "running" && plan.tasks[*j].agent == profile.id)
                .count();
            if active >= profile.max_sessions
                || report
                    .account_restrictions
                    .contains_key(profile.quota_key())
            {
                continue;
            }
            if report.tasks.iter().enumerate().any(|(j, t)| {
                t.status == "waiting_availability"
                    && plan
                        .agents
                        .iter()
                        .find(|a| a.id == plan.tasks[j].agent)
                        .unwrap()
                        .quota_key()
                        == profile.quota_key()
            }) {
                continue;
            }
            if report.tasks[i].session_id.as_ref().is_some_and(|id| {
                report.tasks.iter().enumerate().any(|(j, t)| {
                    j != i
                        && t.status == "running"
                        && t.session_id.as_ref() == Some(id)
                        && plan
                            .agents
                            .iter()
                            .find(|a| a.id == plan.tasks[j].agent)
                            .is_some_and(|a| {
                                a.parser == profile.parser && a.quota_key() == profile.quota_key()
                            })
                })
            }) {
                continue;
            }
            if report.tasks[i].attempt > 0 {
                let artifacts =
                    dir.join(format!("task-{i}-attempt-{}", report.tasks[i].attempt + 1));
                fs::create_dir(&artifacts).map_err(|e| e.to_string())?;
                report.tasks[i].artifacts = artifacts;
            }
            if worktree
                && let Err(error) = prepare_dependencies(
                    &mut report,
                    i,
                    dir,
                    store,
                    &dependency_context,
                    &dependency_snapshots,
                )
            {
                report.tasks[i].attempt += 1;
                report.tasks[i].status = "blocked".into();
                report.tasks[i].error = Some(format!("dependency preparation failed: {error}"));
                fs::write(
                    report.tasks[i].artifacts.join("preparation-error.txt"),
                    &error,
                )
                .map_err(|e| e.to_string())?;
                store.record("dependency_conflict",&serde_json::json!({"task":report.tasks[i].id,"error":error,"artifacts":report.tasks[i].artifacts}),"Dependency preparation failed; worker was not launched")?;
                continue;
            }
            report.tasks[i].dependencies = dependency_context;
            if (profile.permission_mode == "native_callback"
                || profile.cooperative == crate::plan::CooperativeMode::CheckpointV1)
                && report.tasks[i].session_id.is_none()
            {
                report.tasks[i].session_id = Some(crate::native_guard::session_id()?);
                report.tasks[i].session_established = false;
            }
            report.tasks[i].attempt += 1;
            report.tasks[i].status = "running".into();
            report.tasks[i].started_ms = Some(now());
            let agent = plan
                .agents
                .iter()
                .find(|a| a.id == plan.tasks[i].agent)
                .unwrap()
                .clone();
            let task = plan.tasks[i].clone();
            let task_report = report.tasks[i].clone();
            // Persist explicit native identity and running ownership before launch.
            save(store, dir, &report)?;
            jobs.spawn(
                async move { (i, worker(task, agent, task_report, worktree, timeout).await) },
            );
        }
        save(store, dir, &report)?;
        match jobs.join_next().await {
            Some(result) => {
                let (i, task) = result.map_err(|e| e.to_string())?;
                if let Some(restriction) = &task.restriction {
                    let profile = report
                        .plan
                        .agents
                        .iter()
                        .find(|agent| agent.id == report.plan.tasks[i].agent)
                        .unwrap();
                    report
                        .account_restrictions
                        .insert(profile.quota_key().into(), restriction.clone());
                }
                report.tasks[i] = task;
            }
            None => {
                if apply_handoffs(&mut report, dir, store)? {
                    report.state = "running".into();
                    save(store, dir, &report)?;
                    continue;
                }
                if !dir.join("stop.request").exists()
                    && !report.tasks.iter().any(|t| t.status == "stopped")
                    && apply_fallbacks(&mut report, dir, store)?
                {
                    report.state = "running".into();
                    save(store, dir, &report)?;
                    continue;
                }
                break;
            }
        }
    }
    if dir.join("stop.request").exists() {
        report.state = "stopped".into();
        store.record("stop",&serde_json::json!({"request":fs::read_to_string(dir.join("stop.request")).map_err(|e|e.to_string())?}),"Human stop completed; owned processes exited and attempts retained")?;
        store.record(
            "approval",
            &serde_json::Value::Null,
            "Stop revoked candidate approval",
        )?;
        capture_checkpoints(&mut report, dir)?;
        save(store, dir, &report)?;
        return Ok(report);
    }
    if report.tasks.iter().any(|t| t.status != "completed") {
        save(store, dir, &report)?;
        for i in 0..report.tasks.len() {
            if let Some(request) = report.tasks[i].pending_tool.clone() {
                let context = verify_tool_context(
                    dir,
                    &report.tasks[i].id,
                    &request,
                    &std::env::current_exe().map_err(|e| e.to_string())?,
                )
                .await;
                let context = match context {
                    Ok(context) if !dir.join("stop.request").exists() => context,
                    result => {
                        let stopped = dir.join("stop.request").exists();
                        report.tasks[i].status = if stopped { "stopped" } else { "blocked" }.into();
                        report.tasks[i].error = Some(result.err().unwrap_or_else(|| {
                            "Human stop during tool context verification".into()
                        }));
                        report.tasks[i].pending_tool = None;
                        continue;
                    }
                };
                let id = crate::gates::Ledger::open(&dir.join("state.sqlite"))?.request(
                    &context,
                    &request,
                    (now() / 1000) as i64,
                )?;
                report.tasks[i].tool_request = Some(id);
                report.tasks[i].pending_tool = None;
            }
        }
        report.state = if dir.join("stop.request").exists()
            || report.tasks.iter().any(|t| t.status == "stopped")
        {
            "stopped"
        } else if report
            .tasks
            .iter()
            .any(|t| t.status == "waiting_availability")
        {
            "waiting_availability"
        } else if report.tasks.iter().any(|t| t.status == "waiting_approval") {
            "waiting_tool_approval"
        } else if report.tasks.iter().any(|t| t.status == "blocked")
            && !report.tasks.iter().any(|t| t.status == "failed")
        {
            "blocked"
        } else {
            "failed"
        }
        .into();
        capture_checkpoints(&mut report, dir)?;
        save(store, dir, &report)?;
        return Ok(report);
    }
    report.verification_attempt += 1;
    let plan = report.plan.clone();
    let verification = async {
        if git.head()? != base || git.branch()? != report.branch {
            return Err("target Git state changed during execution".into());
        }
        let candidate = if worktree {
            let patches = report
                .tasks
                .iter()
                .map(|t| {
                    Repository::open(&t.cwd)?.diff(&t.base, &t.snapshot.as_ref().unwrap().commit)
                })
                .collect::<Result<Vec<_>, String>>()?;
            git.combine(&base, &patches, &dir.join("integration.index"))?
        } else {
            if !git
                .command(&["diff", "--cached", "--name-only"])?
                .is_empty()
            {
                return Err("agent modified the shared Git index".into());
            }
            git.snapshot(&base, &scopes(&plan), &dir.join("integration.index"))?
        };
        let diff = git.diff(&base, &candidate.commit)?;
        report.diff_hash = Some(digest(&diff));
        fs::write(dir.join("diff.patch"), diff).map_err(|e| e.to_string())?;
        report.candidate = Some(candidate.clone());
        if worktree {
            let cwd = dir.join(format!("integration-{}", report.verification_attempt));
            git.worktree(&cwd, &candidate.commit)?;
            report.verification_cwd = cwd;
        }
        for (i, task) in plan.tasks.iter().enumerate() {
            let process = process::run_until(
                ProcessSpec {
                    executable: task.test_command[0].clone().into(),
                    args: task.test_command[1..].to_vec(),
                    cwd: report.verification_cwd.clone(),
                    input: vec![],
                    log_dir: dir.join(format!("test-{i}-{}", report.verification_attempt)),
                    timeout: Duration::from_secs(timeout),
                    signal_grace: Duration::from_secs(5),
                    environment: Default::default(),
                },
                Some(dir.join("stop.request")),
            )
            .await?;
            let evidence = TestReport {
                task: task.id.clone(),
                exit_code: process.exit_code,
                termination_signal: process.termination_signal,
                timed_out: process.timed_out,
                stdout_hash: file_digest(&process.stdout)?,
                stderr_hash: file_digest(&process.stderr)?,
                stdout: process.stdout,
                stderr: process.stderr,
            };
            let passed = evidence.exit_code == Some(0) && !evidence.timed_out;
            report.tests.push(evidence);
            if !passed {
                return Err(format!("integration test failed: {}", task.id));
            }
        }
        let verification_git = Repository::open(&report.verification_cwd)?;
        let verify_base = if worktree { &candidate.commit } else { &base };
        if verification_git
            .snapshot(verify_base, &scopes(&plan), &dir.join("verification.index"))?
            .tree
            != candidate.tree
        {
            return Err("test commands modified candidate files".into());
        }
        Ok::<_, String>(())
    }
    .await;
    match verification {
        Ok(()) => report.state = "waiting_approval".into(),
        Err(e) => {
            report.state = if dir.join("stop.request").exists() {
                "stopped"
            } else {
                "failed"
            }
            .into();
            report.error = Some(e);
        }
    }
    if dir.join("stop.request").exists() {
        report.state = "stopped".into();
        store.record("stop", &serde_json::json!({"request":fs::read_to_string(dir.join("stop.request")).map_err(|e|e.to_string())?}), "Human stopped integration verification")?;
        store.record(
            "approval",
            &serde_json::Value::Null,
            "Stop revoked candidate approval",
        )?;
    }
    capture_checkpoints(&mut report, dir)?;
    save(store, dir, &report)?;
    Ok(report)
}
