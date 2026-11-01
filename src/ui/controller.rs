use super::*;

mod action;
mod job_monitor;
#[cfg(test)]
#[path = "../../tests/support/controller_job_monitor.rs"]
mod job_monitor_tests;
mod process;
mod review;
pub(super) use action::{Action, ActionRequest};

pub(super) struct Job {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) child: Arc<Mutex<Child>>,
    pub(super) logs: PathBuf,
    pub(super) owner: Ownership,
    pub(super) exit: Option<i32>,
    pub(super) finished: bool,
    pub(super) monitoring: bool,
    pub(super) audit_error: Option<String>,
}
pub(super) struct Controller {
    pub(super) dir: PathBuf,
    pub(super) exe: PathBuf,
    pub(super) job: Arc<Mutex<Option<Job>>>,
    pub(super) event_endpoint: Option<crate::ui::events::EventEndpoint>,
    pub(super) event_surface: crate::ui::events::EventSurface,
}
impl Controller {
    pub(super) fn recovery_preview(
        &self,
        task: String,
        paths: Vec<String>,
        test: Option<Vec<String>>,
    ) -> Result<serde_json::Value, String> {
        workflow::recovery_preview(&self.dir, &task, &paths, test)
    }

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
        if !job.finished && !job.monitoring {
            self.poll_unmonitored(job)?;
        }
        let failed = job.audit_error.is_some() || job.exit != Some(0);
        Ok(
            serde_json::json!({"id":job.id,"kind":job.label,"state":if !job.finished {"running"} else if !failed {"completed"} else {"failed"},"exit_code":job.exit,"termination_signal":job.owner.termination_signal,"error":job.audit_error.clone().unwrap_or(if job.finished && job.exit!=Some(0){tail(&job.logs.join("stderr.log"),4096)?}else{String::new()})}),
        )
    }

    #[cfg(test)]
    fn start_job_monitor(&self) -> Result<(), String> {
        let mut slot = self
            .job
            .lock()
            .map_err(|_| "UI job state is poisoned".to_owned())?;
        let job = slot
            .as_mut()
            .ok_or_else(|| "No UI job to monitor".to_owned())?;
        job_monitor::start(
            Arc::clone(&job.child),
            job.id.clone(),
            Arc::clone(&self.job),
            self.dir.clone(),
        )
        .map_err(|error| error.to_string())?;
        job.monitoring = true;
        Ok(())
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
        if let Action::ToolRefresh { sha } = &request.action {
            let report = workflow::load(&self.dir)?;
            if report.state == "running" {
                if workflow::revision(&report)? != request.revision {
                    return Err(crate::ui::messages::text(
                        "native.run_changed_since_review_refresh_before_acting",
                    )
                    .into());
                }
                workflow::queue_live_tool_refresh(&self.dir, sha)?;
                return Ok(serde_json::json!({
                    "accepted": true,
                    "kind": "tool_refresh",
                    "queued": true,
                    "message": crate::ui::messages::text(
                        "native.tool_refresh_queued_for_active_coordinator",
                    ),
                }));
            }
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
        let background = crate::process::is_background_operation(label);
        let fallback_out = out.try_clone().map_err(|error| error.to_string())?;
        let fallback_err = err.try_clone().map_err(|error| error.to_string())?;
        let mut command = if background {
            crate::subprocess::background_command(&self.exe)
        } else {
            crate::subprocess::command(&self.exe)
        };
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(out)
            .stderr(err);
        if let Some(endpoint) = &self.event_endpoint {
            endpoint.configure(&mut command, self.event_surface);
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        owner.phase = "launching".into();
        record_owner(&logs, &owner)?;
        let child = match command.spawn() {
            Ok(child) => child,
            Err(error) if background && process::breakaway_denied(&error) => {
                let is_in_host_job = match crate::subprocess::current_process_is_in_host_job() {
                    Ok(value) => value,
                    Err(query_error) => {
                        owner.phase = "not_spawned".into();
                        record_owner(&logs, &owner)?;
                        return Err(format!(
                            "Windows denied process breakaway ({error}); host-job membership could not be checked ({query_error}), so background work was not started"
                        ));
                    }
                };
                if !crate::subprocess::background_fallback_allowed(Some(is_in_host_job)) {
                    owner.phase = "not_spawned".into();
                    record_owner(&logs, &owner)?;
                    return Err(format!(
                        "Windows denied process breakaway ({error}); this app is inside a host job, so background work was not started because survival after app close cannot be guaranteed"
                    ));
                }
                let mut fallback = crate::subprocess::background_fallback_command(&self.exe);
                fallback
                    .args(args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::from(fallback_out))
                    .stderr(Stdio::from(fallback_err));
                #[cfg(unix)]
                {
                    use std::os::unix::process::CommandExt;
                    fallback.process_group(0);
                }
                match fallback.spawn() {
                    Ok(child) => child,
                    Err(error) => {
                        owner.phase = "not_spawned".into();
                        record_owner(&logs, &owner)?;
                        return Err(error.to_string());
                    }
                }
            }
            Err(error) => {
                owner.phase = "not_spawned".into();
                record_owner(&logs, &owner)?;
                return Err(error.to_string());
            }
        };
        owner.pid = Some(child.id());
        owner.process_group = Some(child.id() as i32);
        owner.phase = "running".into();
        *slot = Some(Job {
            id: id.clone(),
            label: label.into(),
            child: Arc::new(Mutex::new(child)),
            logs: logs.clone(),
            owner,
            exit: None,
            finished: false,
            monitoring: false,
            audit_error: None,
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
        if let Some(job) = slot.as_mut()
            && let Err(error) = job_monitor::start(
                Arc::clone(&job.child),
                job.id.clone(),
                Arc::clone(&self.job),
                self.dir.clone(),
            )
        {
            eprintln!("could not start coordinator process monitor: {error}");
        } else if let Some(job) = slot.as_mut() {
            job.monitoring = true;
        }
        Ok(serde_json::json!({"accepted":true,"id":id,"kind":label}))
    }
}
