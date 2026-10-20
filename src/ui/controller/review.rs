use super::*;

impl Controller {
    pub(in crate::ui) fn snapshot(
        &self,
        icons: bool,
        quota_refresh_last: &std::collections::BTreeMap<String, u64>,
    ) -> Result<serde_json::Value, String> {
        let job = self.poll()?;
        let availability_status = workflow::ensure_availability_watcher(&self.dir, &self.exe);
        let report = workflow::load(&self.dir)?;
        let revision = workflow::revision(&report)?;
        let value = public_report(&report)?;
        let store = Store::open(self.dir.join("state.sqlite"))?;
        let owned_changes = crate::folder::owned_changes(&store, &report)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;
        let quotas = report
            .plan
            .agents
            .iter()
            .map(|profile| {
                Ok((
                    profile.id.clone(),
                    crate::quota::stored_for_run(&store, profile, now, &self.dir)?,
                ))
            })
            .collect::<Result<serde_json::Map<String, serde_json::Value>, String>>()?;
        let language = crate::ui::language::load();
        let quota_values = serde_json::Value::Object(quotas.clone());
        let quota_window_labels = crate::quota::window_labels(&quota_values, &language);
        let quota_window_reset_labels = crate::quota::window_reset_labels(&quota_values, &language);
        let quota_budget_rows = crate::quota::budget_rows(&quota_values, &language);
        let quota_refresh_due = crate::quota::due_by_profile(
            &report.plan.agents,
            quota_refresh_last,
            (now as u64) * 1000,
        );
        let ledger = crate::gates::Ledger::open(&self.dir.join("state.sqlite"))?;
        let approvals = report
            .tasks
            .iter()
            .filter_map(|t| t.tool_request.as_deref())
            .map(|id| ledger.receipt(id))
            .collect::<Result<Vec<_>, _>>()?;
        let mut approvals: Vec<_> = approvals.into_iter().flatten().collect();
        // DD-32: running workers wait on ledger-only requests the report does not reference.
        for receipt in ledger.requests()? {
            let live = receipt.status == "pending"
                && report.tasks.iter().any(|t| {
                    t.id == receipt.context.task
                        && t.status == "running"
                        && t.session_id.as_deref() == Some(receipt.context.session.as_str())
                        // Requests left by an interrupted earlier attempt are not live.
                        && t.started_ms.is_none_or(|ms| receipt.created_at as u128 * 1000 + 1000 >= ms)
                });
            if live && !approvals.iter().any(|r| r.id == receipt.id) {
                approvals.push(receipt);
            }
        }
        Ok(
        serde_json::json!({"report":value,"revision":revision,"quotas":quotas,"quota_window_labels":quota_window_labels,"quota_window_reset_labels":quota_window_reset_labels,"quota_budget_rows":quota_budget_rows,"quota_refresh_due":quota_refresh_due,"job":job,"tool_requests":approvals,"tool_decision_wait_secs":crate::native_guard::decision_wait(),"run_dir":self.dir,"phase":workflow::phase_status(&self.dir),"owned_changes":owned_changes,"availability_timer":if self.dir.join("stop.request").exists(){vec![]}else{workflow::availability_timer(&report,now)} ,"availability_timer_error":availability_status.err(),"activity":crate::briefing::running_activity(&store,&report,(now as u64)*1000)?}),
        )
        .map(|mut snapshot| {
            if icons {
                snapshot["icons"] = serde_json::json!(crate::icons::icon_map(
                    report.plan.agents.iter().map(|a| a.parser),
                ));
            }
            snapshot
        })
    }
    pub(in crate::ui) fn evidence(&self) -> Result<serde_json::Value, String> {
        let report = workflow::load(&self.dir)?;
        let revision = workflow::revision(&report)?;
        let path = self.dir.join("diff.patch");
        let (diff, truncated) = if path.exists() {
            crate::ui::diff::read(&path, 512 * 1024, report.diff_hash.as_deref())?
        } else {
            if report
                .diff_hash
                .as_ref()
                .is_some_and(|hash| *hash != workflow::digest(b""))
            {
                return Err(crate::ui::messages::text(
                    "native.candidate_diff_changed_verification_required",
                )
                .into());
            }
            (String::new(), false)
        };
        let tests=report.tests.iter().map(|t|Ok(serde_json::json!({"task":t.task,"exit_code":t.exit_code,"timed_out":t.timed_out,"stdout":tail(&t.stdout,8192)?,"stderr":tail(&t.stderr,8192)?}))).collect::<Result<Vec<_>,String>>()?;
        if workflow::revision(&workflow::load(&self.dir)?)? != revision {
            return Err(crate::ui::messages::text(
                "native.run_changed_while_collecting_review_evidence",
            )
            .into());
        }
        Ok(
            serde_json::json!({"revision":revision,"report":public_report(&report)?,"diff":diff,"truncated":truncated,"test_output":tests}),
        )
    }
    /// DD-117: what the coordinator did in this run (claims, verification, reviews, tool
    /// decisions, recovery), and its integration test output for the Raw logs tab.
    pub(in crate::ui) fn coordinator(&self) -> Result<serde_json::Value, String> {
        let report = workflow::load(&self.dir)?;
        let store = Store::open(self.dir.join("state.sqlite"))?;
        let history: Vec<_> = store
            .history(300)?
            .into_iter()
            .map(|mut h| {
                h["payload"] = crate::ui::clip_strings(&h["payload"], 2000);
                h
            })
            .collect();
        let mut stdout = String::new();
        let mut stderr = String::new();
        for test in &report.tests {
            let head = format!(
                "== {} · exit {}{}\n",
                test.task,
                test.exit_code.map_or("?".into(), |c| c.to_string()),
                if test.timed_out { " · timed out" } else { "" }
            );
            stdout.push_str(&head);
            stdout.push_str(&tail(&test.stdout, 8 * 1024).unwrap_or_default());
            stderr.push_str(&head);
            stderr.push_str(&tail(&test.stderr, 8 * 1024).unwrap_or_default());
        }
        Ok(serde_json::json!({"history":history,"stdout":stdout,"stderr":stderr}))
    }
    pub(in crate::ui) fn task(&self, index: usize) -> Result<serde_json::Value, String> {
        let report = workflow::load(&self.dir)?;
        let task = report
            .tasks
            .get(index)
            .ok_or(crate::ui::messages::text("native.unknown_task_index"))?;
        let store = Store::open(self.dir.join("state.sqlite"))?;
        let events = store.task_events(&task.id, task.attempt as u64)?;
        let mut usage = serde_json::Value::Null;
        let events=events.into_iter().rev().take(100).map(|e|{
            if usage.is_null() {usage=e.pointer("/payload/usage").or_else(||e.pointer("/payload/message/usage")).cloned().unwrap_or(serde_json::Value::Null);}
            // DD-112: structured payload for the CLI-like Activity view, long strings clipped.
            serde_json::json!({"seq":e["seq"],"kind":e["kind"],"session_id":e["session_id"],"raw_offset":e["raw_offset"],"payload":clip_strings(&e["payload"],4000)})
        }).collect::<Vec<_>>();
        let logs = task.artifacts.join("process");
        // Tool requests the orchestrator decided for this task, shown in Activity.
        let decisions: Vec<_> = store
            .audit("tool_orchestrator_decision", 200)?
            .into_iter()
            .filter(|d| d["task"] == task.id.as_str())
            .collect();
        Ok(
            serde_json::json!({"task":task,"decisions":decisions,"events":events.into_iter().rev().collect::<Vec<_>>(),"usage":usage,"stdout":if logs.join("stdout.jsonl").exists(){tail(&logs.join("stdout.jsonl"),64*1024)?}else{String::new()},"stderr":if logs.join("stderr.log").exists(){tail(&logs.join("stderr.log"),32*1024)?}else{String::new()},"knowledge":crate::knowledge::status(&self.dir,if report.state=="completed" {report.folder.as_ref().map_or(task.cwd.as_path(),|f|f.original.as_path())}else{task.cwd.as_path()})?}),
        )
    }
}
