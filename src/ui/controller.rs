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
                    return Err("unknown quota agent".into());
                }
                vec!["quota-refresh".into(), agent.clone()]
            }
            Self::Approve { sha } => vec!["approve".into(), sha.clone()],
            Self::Merge { sha } => {
                if report.state != "approved"
                    || report.candidate.as_ref().is_none_or(|c| &c.commit != sha)
                {
                    return Err("merge requires the exact approved candidate".into());
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
            Self::Stop { .. } => return Err("stop is handled directly".into()),
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
    pub(super) fn poll(&self) -> Result<serde_json::Value, String> {
        let mut slot = self.job.lock().map_err(|_| "UI job state poisoned")?;
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
                &format!("UI action {} finished with exit {code:?}, signal {:?}", job.label, job.owner.termination_signal),
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
            serde_json::json!({"report":value,"revision":revision,"quotas":quotas,"job":job,"tool_requests":approvals,"tool_decision_wait_secs":crate::native_guard::decision_wait(),"run_dir":self.dir}),
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
                serde_json::json!({"accepted":true,"kind":"stop","message":"Stop requested; inspect task status until processes are stopped."}),
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
                serde_json::json!({"accepted":true,"kind":request.action.label(),"decided":true,"message":"Decision recorded; the paused worker continues."}),
            );
        }
        self.poll()?;
        let mut slot = self.job.lock().map_err(|_| "UI job state poisoned")?;
        if slot.as_ref().is_some_and(|j| !j.finished) {
            return Err("another UI action is still running".into());
        }
        let report = workflow::load(&self.dir)?;
        if workflow::revision(&report)? != request.revision {
            return Err("run changed since review; refresh before acting".into());
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
            &format!(
                "Human UI action {} accepted; completion remains pending",
                label
            ),
        )?;
        Ok(serde_json::json!({"accepted":true,"id":id,"kind":label}))
    }
    pub(super) fn evidence(&self) -> Result<serde_json::Value, String> {
        let report = workflow::load(&self.dir)?;
        let revision = workflow::revision(&report)?;
        let path = self.dir.join("diff.patch");
        let (diff, truncated) = if path.exists() {
            bounded(&path, 512 * 1024)?
        } else {
            (String::new(), false)
        };
        if !truncated
            && let Some(expected) = &report.diff_hash
            && workflow::digest(diff.as_bytes()) != *expected
        {
            return Err("candidate diff changed; verification required".into());
        }
        let tests=report.tests.iter().map(|t|Ok(serde_json::json!({"task":t.task,"exit_code":t.exit_code,"timed_out":t.timed_out,"stdout":tail(&t.stdout,8192)?,"stderr":tail(&t.stderr,8192)?}))).collect::<Result<Vec<_>,String>>()?;
        if workflow::revision(&workflow::load(&self.dir)?)? != revision {
            return Err("run changed while collecting review evidence".into());
        }
        Ok(
            serde_json::json!({"revision":revision,"report":public_report(&report)?,"diff":diff,"truncated":truncated,"test_output":tests}),
        )
    }
    pub(super) fn task(&self, index: usize) -> Result<serde_json::Value, String> {
        let report = workflow::load(&self.dir)?;
        let task = report.tasks.get(index).ok_or("unknown task index")?;
        let store = Store::open(self.dir.join("state.sqlite"))?;
        let events = store.task_events(&task.id, task.attempt as u64)?;
        let mut usage = serde_json::Value::Null;
        let events=events.into_iter().rev().take(100).map(|e|{
            if usage.is_null() {usage=e.pointer("/payload/usage").or_else(||e.pointer("/payload/message/usage")).cloned().unwrap_or(serde_json::Value::Null);}
            let payload=e["payload"].to_string();serde_json::json!({"seq":e["seq"],"kind":e["kind"],"session_id":e["session_id"],"raw_offset":e["raw_offset"],"text":payload.chars().take(1600).collect::<String>()})
        }).collect::<Vec<_>>();
        let logs = task.artifacts.join("process");
        Ok(
            serde_json::json!({"task":task,"events":events.into_iter().rev().collect::<Vec<_>>(),"usage":usage,"stdout":if logs.join("stdout.jsonl").exists(){tail(&logs.join("stdout.jsonl"),64*1024)?}else{String::new()},"stderr":if logs.join("stderr.log").exists(){tail(&logs.join("stderr.log"),32*1024)?}else{String::new()},"knowledge":crate::knowledge::status(&self.dir,if report.state=="completed" {report.folder.as_ref().map_or(task.cwd.as_path(),|f|f.original.as_path())}else{task.cwd.as_path()})?}),
        )
    }
}
