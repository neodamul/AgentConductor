use super::*;
use crate::{git::Repository, plan::Plan, ui::messages};

impl Startup {
    pub(super) fn revision(&self, s: &Saved) -> Result<String, String> {
        // Observations and scheduler progress cannot invalidate an unrelated edit.
        let mut context = serde_json::to_value(s).map_err(|e| e.to_string())?;
        for key in [
            "quota_job",
            "metadata_queue",
            "metadata_input",
            "metadata_settled",
            "metadata_error",
        ] {
            context.as_object_mut().unwrap().remove(key);
        }
        if let Some(profiles) = context["config"]["profiles"].as_array_mut() {
            for profile in profiles {
                profile.as_object_mut().unwrap().remove("catalog");
            }
        }
        let mut bytes = serde_json::to_vec(&context).map_err(|e| e.to_string())?;
        if let Some(p) = &s.proposal {
            let path = p.directory.join("review.json");
            if path.exists() {
                bytes.extend(storage::bytes(&path)?);
            }
            bytes.extend(
                Repository::open(p.workspace.as_deref().unwrap_or(&s.config.repository))?
                    .head()?
                    .as_bytes(),
            );
        }
        Ok(crate::workflow::digest(&bytes))
    }

    pub(super) fn quota_job(&self, s: &Saved) -> Result<serde_json::Value, String> {
        jobs::state(&self.quota_broker, &s.quota_job)
    }

    pub(super) fn settle(&self, s: &mut Saved, job: &serde_json::Value) -> Result<(), String> {
        if job["state"] != "completed" {
            return Ok(());
        }
        if s.job.as_ref().is_some_and(|j| j.operation == "plan")
            && s.proposal.as_ref().is_some_and(|p| p.phase == "planning")
        {
            let p = s.proposal.as_mut().unwrap();
            let mut plan: Plan = storage::read(&p.directory.join("plan.json"))?;
            if plan.goal != s.config.goal
                || profiles::execution_view(&plan.agents)?
                    != profiles::execution_view(&s.config.profiles)?
            {
                return Err(
                    messages::text("native.planner_changed_registered_profiles_or_goal").into(),
                );
            }
            plan.workspace_mode = s.config.workspace_mode;
            plan.validate()?;
            storage::write(&p.directory.join("review.json"), &plan)?;
            p.phase = "ready".into();
            crate::messenger::notify_plan(
                &self.broker.dir,
                &s.config.messenger,
                &s.config.name,
                &s.config.goal,
                plan.tasks.len(),
            );
            slug::adopt_planned_slug(&mut s.config, &self.broker.dir, p)?;
            if let Some(slug) = &p.slug {
                let root = crate::slugs::root(
                    &s.config.repository,
                    &self.broker.dir,
                    &s.config.name,
                    slug,
                );
                p.slug_phase = crate::slug_progress::progress(&root)["current_phase"]
                    .as_str()
                    .map(str::to_owned);
            }
            self.store(s)?;
        } else if s
            .job
            .as_ref()
            .is_some_and(|j| j.operation.starts_with("catalog:"))
        {
            let profiles: Vec<AgentProfile> =
                storage::read(&self.broker.dir.join("profiles.json"))?;
            validate_profiles(&profiles)?;
            let mut expected = s.config.profiles.clone();
            for (a, b) in expected.iter_mut().zip(&profiles) {
                a.catalog = b.catalog.clone();
            }
            if serde_json::to_vec(&expected).map_err(|e| e.to_string())?
                != serde_json::to_vec(&profiles).map_err(|e| e.to_string())?
            {
                return Err(messages::text(
                    "native.discovery_changed_registered_execution_settings",
                )
                .into());
            }
            s.config.profiles = profiles;
            s.job = None;
            self.store(s)?;
        }
        Ok(())
    }

    pub(super) fn launch(
        &self,
        s: &mut Saved,
        operation: &str,
        args: Vec<String>,
    ) -> Result<serde_json::Value, String> {
        let mut slot = self
            .broker
            .job
            .lock()
            .map_err(|_| messages::text("native.startup_job_state_poisoned"))?;
        if slot.as_ref().is_some_and(|j| !j.finished) {
            return Err(messages::text("native.startup_operation_still_running").into());
        }
        let result = self
            .broker
            .launch_locked(operation, &args, &self.revision(s)?, &mut slot)?;
        s.job = Some(StoredJob {
            id: result["id"]
                .as_str()
                .ok_or(messages::text("native.missing_job_id"))?
                .into(),
            operation: operation.into(),
            started_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis()
                .try_into()
                .map_err(|_| messages::text("native.clock_out_of_range"))?,
        });
        self.store(s)?;
        Ok(result)
    }

    pub(crate) fn poll_job(&self) -> Result<(), String> {
        self.broker.poll()?;
        self.quota_broker.poll().map(|_| ())
    }

    pub(crate) fn root(&self) -> &Path {
        &self.broker.dir
    }

    /// DD-48: name a freshly created workspace through the same validated store.
    pub(crate) fn set_name(&self, name: &str) -> Result<(), String> {
        validate_name(name)?;
        let mut saved = self
            .saved
            .lock()
            .map_err(|_| messages::text("native.startup_settings_poisoned"))?;
        let mut next = saved.clone();
        next.config.name = name.into();
        self.store(&next)?;
        *saved = next;
        Ok(())
    }

    /// Switching workspaces must not orphan a planning or start job.
    pub(crate) fn ensure_idle(&self) -> Result<(), String> {
        let saved = self
            .saved
            .lock()
            .map_err(|_| messages::text("native.startup_settings_poisoned"))?;
        if self.job(&saved)?["state"] == "running" || self.quota_job(&saved)?["state"] == "running"
        {
            return Err(
                messages::text("native.wait_for_the_running_planning_or_metadata_job").into(),
            );
        }
        drop(saved);
        self.ensure_no_live_jobs()
    }

    pub(crate) fn executable(&self) -> &Path {
        &self.broker.exe
    }

    pub(crate) fn active_run(&self) -> Result<Option<PathBuf>, String> {
        let saved = self
            .saved
            .lock()
            .map_err(|_| messages::text("native.startup_settings_poisoned"))?;
        if let Some(proposal) = &saved.proposal
            && proposal.phase == "starting"
            && proposal.run.join("report.json").exists()
        {
            let report = crate::workflow::load(&proposal.run)?;
            if report.plan.goal != saved.config.goal
                || !run_binding::matches(
                    proposal
                        .workspace
                        .as_deref()
                        .unwrap_or(&saved.config.repository),
                    &saved.config.repository,
                    &proposal.repository_base,
                    &report.repo,
                )?
                || report.base != proposal.repository_base
            {
                return Err(
                    messages::text("native.run_does_not_belong_to_the_reviewed_startup").into(),
                );
            }
            return Ok(Some(proposal.run.clone()));
        }
        Ok(self.last_run(&saved))
    }

    pub(super) fn last_run(&self, saved: &Saved) -> Option<PathBuf> {
        let run = saved.last_run.as_ref()?.canonicalize().ok()?;
        let root = self.broker.dir.canonicalize().ok()?;
        (run.starts_with(&root) && run.join("report.json").is_file()).then_some(run)
    }

    pub(super) fn ensure_no_live_jobs(&self) -> Result<(), String> {
        for entry in fs::read_dir(&self.broker.dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if !entry
                .file_name()
                .to_string_lossy()
                .starts_with("coordinator-ui-")
            {
                continue;
            }
            let path = entry.path().join("process/owner.json");
            let owner: crate::process::Ownership = storage::read(&path)?;
            if owner.phase == "terminal"
                && owner.exit_code == Some(0)
                && owner.pid.is_some_and(|id| id > 0 && id <= i32::MAX as u32)
                && owner.process_group == owner.pid.map(|id| id as i32)
            {
                continue;
            }
            owner.ensure_quiescent().map_err(|e| {
                messages::render(
                    "native.retained_startup_job_is_live_or_unknown",
                    &[("e", &e.to_string())],
                )
            })?;
        }
        Ok(())
    }
}
