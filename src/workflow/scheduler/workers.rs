use super::*;

pub(super) async fn run_workers(
    mut report: Report,
    dir: &Path,
    store: &mut Store,
    timeout: u64,
    worktree: bool,
) -> Result<Report, String> {
    let mut jobs = tokio::task::JoinSet::new();
    loop {
        super::availability_timer::sync_availability(
            &mut report,
            dir,
            store,
            (now() / 1000) as i64,
        )?;
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
                super::super::assignment::active_feedback::settle(store, &mut report, i)?;
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
    Ok(report)
}
