use super::*;

impl Startup {
    pub fn dispatch(&self, bytes: &[u8]) -> Result<serde_json::Value, String> {
        if bytes.len() > 64 * 1024 {
            return Err("native request exceeds 64 KiB".into());
        }
        let request: Request = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        let mut saved = self.saved.lock().map_err(|_| "startup settings poisoned")?;
        let job = self.job(&saved)?;
        self.settle(&mut saved, &job)?;
        if let Request::Snapshot {} = request {
            let mut config = serde_json::to_value(&saved.config).map_err(|e| e.to_string())?;
            for p in config["profiles"]
                .as_array_mut()
                .ok_or("invalid registry")?
            {
                p.as_object_mut().unwrap().remove("environment");
            }
            let proposal = if let Some(p) = &saved.proposal {
                let path = p.directory.join("review.json");
                let mut value = serde_json::to_value(p).map_err(|e| e.to_string())?;
                if path.exists() {
                    let plan: Plan = read(&path)?;
                    let digest =
                        workflow::digest(&serde_json::to_vec(&plan).map_err(|e| e.to_string())?);
                    let mut public = serde_json::to_value(plan).map_err(|e| e.to_string())?;
                    for agent in public["agents"].as_array_mut().unwrap() {
                        agent.as_object_mut().unwrap().remove("environment");
                    }
                    value["plan"] = public;
                    value["digest"] = digest.into();
                }
                value
            } else {
                serde_json::Value::Null
            };
            return Ok(
                serde_json::json!({"revision":self.revision(&saved)?,"config":config,"proposal":proposal,"job":job,"root":self.broker.dir,"templates":defaults()?.profiles,"run_available":saved.proposal.as_ref().is_some_and(|p|p.phase=="starting" && p.run.join("report.json").exists()) || self.last_run(&saved).is_some(),"icons":crate::icons::icon_map(saved.config.profiles.iter().map(|p|p.parser))}),
            );
        }
        if let Request::BaselineStatus {} = request {
            return self.baseline_status(&saved);
        }
        if let Request::PreviewSettings { folder } = &request {
            return self.preview_settings(&saved, folder);
        }
        if job["state"] == "running" {
            return Err("startup operation still running".into());
        }
        self.ensure_no_live_jobs()?;
        let revision = match &request {
            Request::Save { revision, .. }
            | Request::Discover { revision, .. }
            | Request::Catalog { revision, .. }
            | Request::Prepare { revision }
            | Request::Draft { revision }
            | Request::ReadPlan { revision, .. }
            | Request::CommitBaseline { revision }
            | Request::IgnoreOsFiles { revision }
            | Request::SaveProfiles { revision, .. }
            | Request::ExportSettings { revision, .. }
            | Request::ImportSettings { revision, .. }
            | Request::Edit { revision, .. }
            | Request::Start { revision, .. } => revision,
            Request::Snapshot {} | Request::BaselineStatus {} | Request::PreviewSettings { .. } => {
                unreachable!()
            }
        };
        if revision != &self.revision(&saved)? {
            return Err("startup state changed since review; refresh before acting".into());
        }
        match request {
            Request::Save { mut config, .. } => {
                validate_profiles(&config.profiles)?;
                if !config.profiles.iter().any(|p| p.id == config.coordinator) {
                    return Err("coordinator is not registered".into());
                }
                validate_name(&config.name)?;
                crate::roles::validate(&config.roles, &config.profiles)?;
                if config.goal.len() > 16 * 1024 {
                    return Err("goal exceeds 16 KiB".into());
                }
                if !config.repository.as_os_str().is_empty() {
                    config.repository = Repository::open(&config.repository)?.root;
                    if self.broker.dir.starts_with(&config.repository) {
                        return Err("startup artifacts must be outside repository".into());
                    }
                }
                profiles::carry_over(&mut config.profiles, &saved.config.profiles);
                // DD-58: keep a reviewed plan unless what it was reviewed against changed.
                let keep = config.goal == saved.config.goal
                    && config.repository == saved.config.repository
                    && serde_json::to_value(config.workspace_mode).ok()
                        == serde_json::to_value(saved.config.workspace_mode).ok()
                    && profiles::execution_view(&config.profiles)?
                        == profiles::execution_view(&saved.config.profiles)?;
                let next = Saved {
                    config,
                    proposal: if keep { saved.proposal.clone() } else { None },
                    job: if keep { saved.job.clone() } else { None },
                    last_run: saved.last_run.clone(),
                };
                self.store(&next)?;
                *saved = next;
                Ok(serde_json::json!({"saved":true}))
            }
            Request::Catalog { agent, catalog, .. } => {
                validate_catalog(&catalog)?;
                let mut next = saved.clone();
                next.config
                    .profiles
                    .iter_mut()
                    .find(|p| p.id == agent)
                    .ok_or("unknown agent")?
                    .catalog = Some(catalog);
                // DD-57: observed models are metadata; a reviewed plan stays valid.
                next.job = None;
                self.store(&next)?;
                *saved = next;
                Ok(serde_json::json!({"saved":true}))
            }
            Request::Discover { agent, .. } => {
                let profile = saved
                    .config
                    .profiles
                    .iter()
                    .find(|p| p.id == agent)
                    .ok_or("unknown agent")?;
                if profile.parser == crate::protocol::Provider::Custom {
                    return Err("custom agents import an observed catalog".into());
                }
                write(
                    &self.broker.dir.join("profiles.json"),
                    &saved.config.profiles,
                )?;
                let discovery = self
                    .broker
                    .dir
                    .join(format!("models-{}", crate::native_guard::session_id()?));
                self.launch(
                    &mut saved,
                    &format!("catalog:{agent}"),
                    vec![
                        "models-discover".into(),
                        self.broker
                            .dir
                            .join("profiles.json")
                            .to_string_lossy()
                            .into(),
                        agent,
                        discovery.to_string_lossy().into(),
                    ],
                )
            }
            Request::Prepare { .. } => self.start_planning(&mut saved, None),
            Request::ReadPlan { path, .. } => self.start_planning(&mut saved, Some(&path)),
            Request::CommitBaseline { .. } => self.commit_baseline(&saved),
            Request::IgnoreOsFiles { .. } => self.ignore_os_files(&saved),
            Request::SaveProfiles { profiles, .. } => self.save_profiles(&mut saved, profiles),
            Request::ExportSettings { folder, .. } => self.export_settings(&saved, &folder),
            Request::ImportSettings {
                folder,
                keep_verified,
                ..
            } => self.import_settings(&mut saved, &folder, keep_verified),
            // DD-45: a hand-written plan starts from an empty form; no coordinator or job.
            Request::Draft { .. } => {
                let config = &saved.config;
                if config.goal.trim().is_empty() {
                    return Err("enter a nonempty goal".into());
                }
                validate_profiles(&config.profiles)?;
                let agent = &config
                    .profiles
                    .first()
                    .ok_or("register an agent before writing a plan")?
                    .id;
                let repo = Repository::open(&config.repository)?;
                repo.clean()?;
                if self.broker.dir.starts_with(&repo.root) {
                    return Err("startup artifacts must be outside repository".into());
                }
                // DD-55: with role rules the draft starts as development, testing, documentation.
                let tasks = ["development", "testing", "documentation"]
                    .iter()
                    .enumerate()
                    .map(|(i, role)| {
                        let ruled = config.roles.get(*role).and_then(|a| a.first());
                        Task {
                            id: format!("task-{}", i + 1),
                            agent: ruled.map_or(agent, |r| &r.agent).clone(),
                            prompt: String::new(),
                            paths: vec![],
                            depends_on: vec![],
                            test_command: vec![],
                            model: ruled.and_then(|r| r.model.clone()),
                            role: ruled.map(|_| (*role).to_string()),
                        }
                    })
                    .collect();
                let plan = Plan {
                    goal: config.goal.clone(),
                    agents: config.profiles.clone(),
                    tasks,
                    workspace_mode: config.workspace_mode,
                };
                let directory = self
                    .broker
                    .dir
                    .join(format!("plan-{}", crate::native_guard::session_id()?));
                fs::create_dir(&directory).map_err(|e| e.to_string())?;
                write(&directory.join("review.json"), &plan)?;
                saved.proposal = Some(Proposal {
                    directory,
                    repository_base: repo.head()?,
                    run: self
                        .broker
                        .dir
                        .join(format!("run-{}", crate::native_guard::session_id()?)),
                    phase: "ready".into(),
                });
                saved.job = None;
                self.store(&saved)?;
                Ok(serde_json::json!({"drafted":true}))
            }
            Request::Edit {
                tasks,
                workspace_mode,
                ..
            } => {
                let p = saved.proposal.as_ref().ok_or("no generated plan")?;
                if p.phase != "ready" {
                    return Err("plan is not ready for review".into());
                }
                let plan = Plan {
                    goal: saved.config.goal.clone(),
                    agents: saved.config.profiles.clone(),
                    tasks,
                    workspace_mode,
                };
                plan.validate()?;
                write(&p.directory.join("review.json"), &plan)?;
                saved.config.workspace_mode = workspace_mode;
                self.store(&saved)?;
                Ok(serde_json::json!({"saved":true}))
            }
            Request::Start { digest, revision } => {
                let p = saved.proposal.as_ref().ok_or("no generated plan")?;
                if p.phase != "ready" {
                    return Err("plan is not ready for execution consent".into());
                }
                let path = p.directory.join("review.json");
                let plan: Plan = read(&path)?;
                plan.validate()?;
                // Executables, arguments and permissions must match; catalogs may be newer.
                if plan.goal != saved.config.goal
                    || profiles::execution_view(&plan.agents)?
                        != profiles::execution_view(&saved.config.profiles)?
                {
                    return Err("reviewed plan changed the registered profiles or goal".into());
                }
                if workflow::digest(&serde_json::to_vec(&plan).map_err(|e| e.to_string())?)
                    != digest
                {
                    return Err("approve the exact reviewed plan digest".into());
                }
                workflow::validate_permissions(&plan)?;
                let repo = Repository::open(&saved.config.repository)?;
                repo.clean()?;
                if repo.head()? != p.repository_base {
                    return Err("repository changed since planning; generate a new plan".into());
                }
                crate::state::Store::open(self.broker.dir.join("state.sqlite"))?.record("plan_execution_consent",&serde_json::json!({"digest":digest,"revision":revision,"repository_base":p.repository_base,"plan":path}),"Human approved the exact reviewed startup plan")?;
                let run = p.run.clone();
                let args = vec![
                    "startup-run".into(),
                    path.to_string_lossy().into(),
                    repo.root.to_string_lossy().into(),
                    run.to_string_lossy().into(),
                    digest,
                    p.repository_base.clone(),
                ];
                saved.proposal.as_mut().unwrap().phase = "starting".into();
                saved.last_run = Some(run.clone());
                self.store(&saved)?;
                let mut result = self.launch(&mut saved, "start", args)?;
                result["run"] = serde_json::json!(run);
                Ok(result)
            }
            Request::Snapshot {} | Request::BaselineStatus {} | Request::PreviewSettings { .. } => {
                unreachable!()
            }
        }
    }
}
