use super::*;

impl Startup {
    pub fn open(root: &Path, exe: &Path) -> Result<Self, String> {
        if !root.exists() {
            fs::create_dir_all(root).map_err(|e| e.to_string())?;
        }
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
                .map_err(|e| e.to_string())?;
        }
        let lease = crate::leases::Lease::acquire(&root.join("startup.lock"))?;
        let settings = root.join("startup.json");
        let mut saved: Saved = if settings.exists() {
            read(&settings)?
        } else {
            let s = Saved {
                imports: None,
                mcp_result: None,
                chat_result: None,
                optimize_result: None,
                optimize_goal: None,
                // DD-72: no pre-registered agents; templates stay available in + Add.
                config: Config {
                    profiles: vec![],
                    coordinator: String::new(),
                    ..defaults()?
                },
                proposal: None,
                job: None,
                quota_job: None,
                metadata_queue: Vec::new(),
                metadata_input: None,
                metadata_settled: None,
                metadata_error: None,
                last_run: None,
            };
            write(&settings, &s)?;
            s
        };
        master::adopt(&root, &mut saved.config.profiles);
        saved.last_run.as_deref().map(crate::workflow::sync_phase);
        // A reviewed plan whose private folder copy is gone (for example after the legacy
        // folder-history copies were removed) can no longer start; it is finished history.
        if saved
            .proposal
            .as_ref()
            .and_then(|p| p.workspace.as_ref())
            .is_some_and(|w| !w.exists())
        {
            saved.proposal = None;
            write(&settings, &saved)?;
        }
        let (custom_profiles, custom_registry_invalid) =
            match crate::custom_profile_registry::CustomProfileRegistry::load_global() {
                Ok(profiles) => (profiles, false),
                Err(_) => (Vec::new(), true),
            };
        let startup = Self {
            quota_broker: Controller {
                dir: root.join("quota-jobs"),
                exe: exe.canonicalize().map_err(|e| e.to_string())?,
                job: Mutex::new(None),
            },
            broker: Controller {
                dir: root,
                exe: exe.canonicalize().map_err(|e| e.to_string())?,
                job: Mutex::new(None),
            },
            saved: Mutex::new(saved),
            custom_profiles,
            custom_registry_invalid,
            _lease: lease,
        };
        if let Ok(saved) = startup.saved.lock() {
            let _ = startup.clear_cache(&saved, None, 8);
            let _ = startup.prune_history(&saved);
        }
        startup.recover_last_run()?;
        Ok(startup)
    }
    pub(super) fn revision(&self, s: &Saved) -> Result<String, String> {
        self.state_revision(s, true)
    }
    pub(super) fn chat_revision(&self, s: &Saved) -> Result<String, String> {
        self.state_revision(s, false)
    }
    fn state_revision(&self, s: &Saved, include_head: bool) -> Result<String, String> {
        // Observations and scheduler progress cannot invalidate an unrelated edit.
        let mut context = serde_json::to_value(s).map_err(|e| e.to_string())?;
        for key in [
            "quota_job",
            "metadata_queue",
            "metadata_input",
            "metadata_settled",
            "metadata_error",
            "chat_result",
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
                bytes.extend(self::bytes(&path)?);
            }
            if include_head {
                bytes.extend(
                    Repository::open(p.workspace.as_deref().unwrap_or(&s.config.repository))?
                        .head()?
                        .as_bytes(),
                );
            }
        }
        Ok(workflow::digest(&bytes))
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
            let mut plan: Plan = read(&p.directory.join("plan.json"))?;
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
            write(&p.directory.join("review.json"), &plan)?;
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
            let profiles: Vec<AgentProfile> = read(&self.broker.dir.join("profiles.json"))?;
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
        let revision = self.revision(s)?;
        self.launch_with_revision(s, operation, args, &revision)
    }
    pub(super) fn launch_with_revision(
        &self,
        s: &mut Saved,
        operation: &str,
        args: Vec<String>,
        revision: &str,
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
            .launch_locked(operation, &args, revision, &mut slot)?;
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
}
