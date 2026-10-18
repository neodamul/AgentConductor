use super::*;

mod review;

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
            Self::Resume {} if report.state == "failed" && !workflow::repairable(report) => {
                return Err(crate::ui::messages::text(
                    "native.failed_run_cannot_be_resumed_revise_or_reassign",
                )
                .into());
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
}
