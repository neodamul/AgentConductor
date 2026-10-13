use super::*;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ActionRequest {
    pub(super) revision: String,
    pub(super) action: Action,
}
#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Action {
    Quota { agent: String },
    Approve { sha: String },
    Merge { sha: String },
    Reject { reason: String },
    Revise { task: String, feedback: String },
    Reassign { task: String, agent: String },
    Model { task: String, model: String },
    Schedule { task: String, at: String },
    ToolApprove { sha: String },
    ToolReject { sha: String },
    ToolRefresh { sha: String },
    Resume {},
    Recover {},
    Stop { reason: String },
}
impl Action {
    pub(super) fn label(&self) -> &'static str {
        match self {
            Self::Quota { .. } => "quota",
            Self::Approve { .. } => "approve",
            Self::Merge { .. } => "merge",
            Self::Reject { .. } => "reject",
            Self::Revise { .. } => "revise",
            Self::Reassign { .. } => "reassign",
            Self::Model { .. } => "model",
            Self::Schedule { .. } => "schedule",
            Self::ToolApprove { .. } => "tool_approve",
            Self::ToolReject { .. } => "tool_reject",
            Self::ToolRefresh { .. } => "tool_refresh",
            Self::Resume {} => "resume",
            Self::Recover {} => "recover",
            Self::Stop { .. } => "stop",
        }
    }
    pub(super) fn args(&self, report: &Report, dir: &Path) -> Result<Vec<String>, String> {
        Ok(match self {
            Self::Quota { agent } => {
                if !report
                    .plan
                    .agents
                    .iter()
                    .any(|profile| &profile.id == agent)
                {
                    return Err(crate::ui::messages::text("native.unknown_quota_agent").into());
                }
                vec!["quota-refresh".into(), agent.clone()]
            }
            Self::Approve { sha } => vec!["approve".into(), sha.clone()],
            Self::Merge { sha } => {
                if report.state != "approved"
                    || report.candidate.as_ref().is_none_or(|c| &c.commit != sha)
                {
                    return Err(crate::ui::messages::text(
                        "native.merge_requires_the_exact_approved_candidate",
                    )
                    .into());
                }
                vec!["merge".into()]
            }
            Self::Reject { reason } => vec!["reject".into(), reason.clone()],
            Self::Revise { task, feedback } => {
                vec!["revise".into(), task.clone(), feedback.clone()]
            }
            Self::Reassign { task, agent } => vec!["reassign".into(), task.clone(), agent.clone()],
            Self::Model { task, model } => {
                vec!["session-model".into(), task.clone(), model.clone()]
            }
            Self::Schedule { task, at } => vec!["schedule".into(), task.clone(), at.clone()],
            Self::ToolApprove { sha } => vec!["tool-approve".into(), sha.clone()],
            Self::ToolReject { sha } => vec!["tool-reject".into(), sha.clone()],
            Self::ToolRefresh { sha } => vec!["tool-refresh".into(), sha.clone()],
            Self::Resume {} if report.state == "stopped" => {
                vec!["ui-resume".into(), workflow::stop_digest(dir)?]
            }
            Self::Resume {} => vec!["resume-due".into()],
            Self::Recover {} => vec!["recover".into()],
            Self::Stop { .. } => {
                return Err(crate::ui::messages::text("native.stop_is_handled_directly").into());
            }
        })
    }
}

pub(super) struct Job {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) child: Child,
    pub(super) logs: PathBuf,
    pub(super) owner: Ownership,
    pub(super) exit: Option<i32>,
    pub(super) finished: bool,
}
pub(super) struct Controller {
    pub(super) dir: PathBuf,
    pub(super) exe: PathBuf,
    pub(super) job: Mutex<Option<Job>>,
}
impl Controller {
    /// DD-135: a UI job of this run is still running (unknown state counts as busy).
    pub(super) fn busy(&self) -> bool {
        self.poll()
            .map(|job| job["state"] == "running")
            .unwrap_or(true)
    }
    pub(super) fn poll(&self) -> Result<serde_json::Value, String> {
        let mut slot = self
            .job
            .lock()
            .map_err(|_| crate::ui::messages::text("native.ui_job_state_poisoned"))?;
        let Some(job) = slot.as_mut() else {
            return Ok(serde_json::Value::Null);
        };
        if !job.finished
            && let Some(status) = job.child.try_wait().map_err(|e| e.to_string())?
        {
            let code = status.code();
            job.exit = code;
            job.finished = true;
            job.owner.phase = "terminal".into();
            job.owner.exit_code = code;
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                job.owner.termination_signal = status.signal();
            }
            record_owner(&job.logs, &job.owner)?;
            Store::open(self.dir.join("state.sqlite"))?.record(
                "ui_action_result",
                &serde_json::json!({"id":job.id,"kind":job.label,"exit_code":code,"termination_signal":job.owner.termination_signal}),
                &crate::ui::messages::render("native.ui_action_finished_with_exit_signal", &[("value1", &job.label.to_string()), ("code", &format!("{:?}", code)), ("value2", &format!("{:?}", job.owner.termination_signal))]),
            )?;
        }
        Ok(
            serde_json::json!({"id":job.id,"kind":job.label,"state":if !job.finished {"running"} else if job.exit==Some(0) {"completed"} else {"failed"},"exit_code":job.exit,"termination_signal":job.owner.termination_signal,"error":if job.finished && job.exit!=Some(0){tail(&job.logs.join("stderr.log"),4096)?}else{String::new()}}),
        )
    }
    pub(super) fn snapshot(&self, icons: bool) -> Result<serde_json::Value, String> {
        let job = self.poll()?;
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
            serde_json::json!({"report":value,"revision":revision,"quotas":quotas,"job":job,"tool_requests":approvals,"tool_decision_wait_secs":crate::native_guard::decision_wait(),"run_dir":self.dir,"phase":workflow::phase_status(&self.dir),"owned_changes":owned_changes}),
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
    pub(super) fn action(&self, request: ActionRequest) -> Result<serde_json::Value, String> {
        if let Action::Stop { reason } = &request.action {
            workflow::stop(&self.dir, reason)?;
            return Ok(
                serde_json::json!({"accepted":true,"kind":"stop","message":crate::ui::messages::text("native.stop_requested_inspect_task_status_until_processes_are")}),
            );
        }
        if let Action::ToolApprove { sha } | Action::ToolReject { sha } = &request.action
            && workflow::load(&self.dir)?.state == "running"
        {
            // DD-32: a paused worker's request is decided at once, bound to its exact
            // SHA, without waiting for another UI job (such as the resumed coordinator).
            let allow = matches!(request.action, Action::ToolApprove { .. });
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?
                .block_on(workflow::decide_live_request(
                    &self.dir, sha, allow, 300, &self.exe,
                ))?;
            return Ok(
                serde_json::json!({"accepted":true,"kind":request.action.label(),"decided":true,"message":crate::ui::messages::text("native.decision_recorded_the_paused_worker_continues")}),
            );
        }
        self.poll()?;
        let mut slot = self
            .job
            .lock()
            .map_err(|_| crate::ui::messages::text("native.ui_job_state_poisoned"))?;
        if slot.as_ref().is_some_and(|j| !j.finished) {
            return Err(
                crate::ui::messages::text("native.another_ui_action_is_still_running").into(),
            );
        }
        let report = workflow::load(&self.dir)?;
        if workflow::revision(&report)? != request.revision {
            return Err(crate::ui::messages::text(
                "native.run_changed_since_review_refresh_before_acting",
            )
            .into());
        }
        let args = request.action.args(&report, &self.dir)?;
        let mut command = vec![
            "ui-action".into(),
            self.dir.to_string_lossy().into_owned(),
            request.revision.clone(),
        ];
        command.extend(args);
        self.launch_locked(
            request.action.label(),
            &command,
            &request.revision,
            &mut slot,
        )
    }
    pub(super) fn launch_locked(
        &self,
        label: &str,
        args: &[String],
        revision: &str,
        slot: &mut Option<Job>,
    ) -> Result<serde_json::Value, String> {
        let id = crate::native_guard::session_id()?;
        let root = self.dir.join(format!("coordinator-ui-{id}"));
        let logs = root.join("process");
        fs::create_dir(&root)
            .and_then(|_| fs::create_dir(&logs))
            .map_err(|e| e.to_string())?;
        let mut owner = Ownership {
            coordinator_pid: std::process::id(),
            pid: None,
            process_group: None,
            phase: "not_spawned".into(),
            exit_code: None,
            termination_signal: None,
            stopped: false,
            timed_out: false,
        };
        record_owner(&logs, &owner)?;
        let out = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(logs.join("stdout.log"))
            .map_err(|e| e.to_string())?;
        let err = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(logs.join("stderr.log"))
            .map_err(|e| e.to_string())?;
        let mut command = crate::subprocess::command(&self.exe);
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(out)
            .stderr(err);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        owner.phase = "launching".into();
        record_owner(&logs, &owner)?;
        let child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
                owner.phase = "not_spawned".into();
                record_owner(&logs, &owner)?;
                return Err(e.to_string());
            }
        };
        owner.pid = Some(child.id());
        owner.process_group = Some(child.id() as i32);
        owner.phase = "running".into();
        *slot = Some(Job {
            id: id.clone(),
            label: label.into(),
            child,
            logs: logs.clone(),
            owner,
            exit: None,
            finished: false,
        });
        record_owner(&logs, &slot.as_ref().unwrap().owner)?;
        Store::open(self.dir.join("state.sqlite"))?.record(
            "ui_action",
            &serde_json::json!({"id":id,"kind":label,"revision":revision}),
            &crate::ui::messages::render(
                "native.human_ui_action_accepted_completion_remains_pending",
                &[("value1", label)],
            ),
        )?;
        Ok(serde_json::json!({"accepted":true,"id":id,"kind":label}))
    }
    pub(super) fn evidence(&self) -> Result<serde_json::Value, String> {
        let report = workflow::load(&self.dir)?;
        let revision = workflow::revision(&report)?;
        let path = self.dir.join("diff.patch");
        let (diff, truncated) = if path.exists() {
            super::diff::read(&path, 512 * 1024, report.diff_hash.as_deref())?
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
    pub(super) fn coordinator(&self) -> Result<serde_json::Value, String> {
        let report = workflow::load(&self.dir)?;
        let store = Store::open(self.dir.join("state.sqlite"))?;
        let history: Vec<_> = store
            .history(300)?
            .into_iter()
            .map(|mut h| {
                h["payload"] = super::clip_strings(&h["payload"], 2000);
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
    pub(super) fn task(&self, index: usize) -> Result<serde_json::Value, String> {
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
