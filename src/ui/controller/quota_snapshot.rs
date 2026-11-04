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
        let quota_indicators =
            crate::quota::indicators(&quota_values, &report.plan.agents, &language, now);
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
            serde_json::json!({"report":value,"revision":revision,"interruptions":workflow::interruptions(&store)?,"activity":crate::briefing::running_activity(&store, &report, (now as u64) * 1000)?,"quotas":quotas,"quota_window_labels":quota_window_labels,"quota_indicators":quota_indicators,"quota_window_reset_labels":quota_window_reset_labels,"quota_budget_rows":quota_budget_rows,"quota_refresh_due":quota_refresh_due,"job":job,"tool_requests":approvals,"coordinator":crate::native_guard::orchestrator_profile(&self.dir).map(|p| p.id),"decision_count":workflow::decisions::count(&self.dir)?,"tool_decision_wait_secs":crate::native_guard::decision_wait(),"run_dir":self.dir,"phase":workflow::phase_status(&self.dir),"owned_changes":owned_changes,"availability_timer":if self.dir.join("stop.request").exists(){vec![]}else{workflow::availability_timer(&report,now)} ,"availability_timer_error":availability_status.err(),"runner":crate::briefing::runner(&self.dir,&report),"stopping":self.dir.join("stop.request").exists(),"verification":crate::briefing::verification(&self.dir,&report)}),
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
}
