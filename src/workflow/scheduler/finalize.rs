use super::*;

pub(super) async fn finish(
    mut report: Report,
    dir: &Path,
    store: &mut Store,
    timeout: u64,
    git: &Repository,
    base: String,
    worktree: bool,
) -> Result<Report, String> {
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
                let mut ledger = crate::gates::Ledger::open(&dir.join("state.sqlite"))?;
                let existing = ledger.requests()?.into_iter().find(|receipt| {
                    receipt.status == "pending"
                        && receipt.context.task == report.tasks[i].id
                        && Some(receipt.context.session.as_str())
                            == report.tasks[i].session_id.as_deref()
                        && receipt.request == request
                });
                let id = match existing {
                    Some(receipt) if receipt.context == context => receipt.id,
                    Some(receipt) => {
                        ledger.supersede(&receipt.id)?;
                        ledger.request(&context, &request, (now() / 1000) as i64)?
                    }
                    None => ledger.request(&context, &request, (now() / 1000) as i64)?,
                };
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
            report.error = report
                .tasks
                .iter()
                .find(|task| task.status == "blocked")
                .and_then(|task| task.error.clone());
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
    // GitHub #273: the run screen reads the attempt to show which integration test runs.
    save(store, dir, &report)?;
    let plan = report.plan.clone();
    if !worktree {
        super::stray::move_stray_files(git, &plan, dir, store)?;
    }
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
        // A project without its own .git keeps its history outside; test commands that call
        // git (for example `git diff --check`) are pointed at that history.
        let test_environment = if worktree {
            Default::default()
        } else {
            git.external_environment()
        };
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
                    environment: test_environment.clone(),
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
        // DD-120: a changed verification report that records a blocking issue fails too.
        for path in candidate
            .changed
            .iter()
            .filter(|p| p.to_lowercase().ends_with(".md"))
        {
            let Ok(text) = fs::read_to_string(report.verification_cwd.join(path)) else {
                continue;
            };
            let Some(findings) = super::blockers::blocking_findings(&text) else {
                continue;
            };
            // The issue is for the task owning the files it names, else the report's owner.
            let owns = |t: &crate::plan::Task, p: &str| {
                t.paths
                    .iter()
                    .any(|s| p == s || p.starts_with(&format!("{}/", s.trim_end_matches('/'))))
            };
            // Reports often name only a file ("app.js"), so a path's file name counts too.
            let names = |t: &crate::plan::Task| {
                t.paths.iter().any(|s| {
                    let file = s.rsplit('/').next().unwrap_or(s);
                    findings.contains(s.as_str()) || (file.contains('.') && findings.contains(file))
                })
            };
            let named = plan.tasks.iter().find(|t| !owns(t, path) && names(t));
            let owner = named
                .or_else(|| plan.tasks.iter().find(|t| owns(t, path)))
                .map_or_else(|| plan.tasks[0].id.clone(), |t| t.id.clone());
            let log = dir.join(format!(
                "verification-report-{}",
                report.verification_attempt
            ));
            fs::create_dir_all(&log).map_err(|e| e.to_string())?;
            let (stdout, stderr) = (log.join("stdout.jsonl"), log.join("stderr.log"));
            fs::write(&stdout, b"").map_err(|e| e.to_string())?;
            fs::write(
                &stderr,
                format!("Blocking issue recorded in {path}:\n{findings}\n"),
            )
            .map_err(|e| e.to_string())?;
            report.tests.push(TestReport {
                task: owner.clone(),
                exit_code: Some(1),
                termination_signal: None,
                timed_out: false,
                stdout_hash: file_digest(&stdout)?,
                stderr_hash: file_digest(&stderr)?,
                stdout,
                stderr,
            });
            return Err(format!(
                "integration verification failed: {path} records a blocking issue for {owner}"
            ));
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
