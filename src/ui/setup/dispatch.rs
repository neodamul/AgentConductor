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
        if let Request::Snapshot { icons, mcp_result } = request {
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
            let mut snapshot = serde_json::json!({"revision":self.revision(&saved)?,"config":config,"proposal":proposal,"job":job,"quota_job":self.quota_job(&saved)?,"root":self.broker.dir,"templates":defaults()?.profiles,"run_available":saved.proposal.as_ref().is_some_and(|p|p.phase=="starting" && p.run.join("report.json").exists()) || self.last_run(&saved).is_some(),"quotas":self.quota_snapshot(&saved)?,"slugs":self.slug_snapshot(&saved),"plan_rules_global":crate::plan_rules::current(None),"language":crate::ui::language::load(),"mcp_result":if mcp_result {self.mcp_result(&saved,&job)?}else{serde_json::Value::Null},"imports":saved.imports});
            if icons {
                snapshot["icons"] = serde_json::json!(crate::icons::icon_map(
                    saved.config.profiles.iter().map(|p| p.parser),
                ));
            }
            return Ok(snapshot);
        }
        if let Request::BaselineStatus {} = request {
            return self.baseline_status(&saved);
        }
        if let Request::PreviewSettings { folder } = &request {
            return self.preview_settings(&saved, folder);
        }
        if let Request::Messenger {} = request {
            return Ok(crate::messenger::status(
                &self.broker.dir,
                &saved.config.messenger,
            ));
        }
        if let Request::TestMessenger { settings, url } = &request {
            let name = saved.config.name.clone();
            drop(saved);
            crate::messenger::test(&self.broker.dir, settings, url.as_deref(), &name)?;
            return Ok(serde_json::json!({"sent":true}));
        }
        if let Request::Quota { agent, revision } = &request {
            if revision != &self.revision(&saved)? {
                return Err("startup state changed since review; refresh before acting".into());
            }
            return self.launch_quota(&mut saved, agent);
        }
        if job["state"] == "running" {
            return Err("startup operation still running".into());
        }
        self.ensure_no_live_jobs()?;
        let revision = match &request {
            Request::McpSave { revision, .. }
            | Request::McpDiscover { revision, .. }
            | Request::McpPreview { revision, .. }
            | Request::McpApply { revision, .. }
            | Request::Save { revision, .. }
            | Request::Quota { revision, .. }
            | Request::Discover { revision, .. }
            | Request::Catalog { revision, .. }
            | Request::Prepare { revision, .. }
            | Request::NextPhase { revision }
            | Request::SavePlanRules { revision, .. }
            | Request::Draft { revision }
            | Request::ReadPlan { revision, .. }
            | Request::CommitBaseline { revision }
            | Request::IgnoreOsFiles { revision }
            | Request::CreateSlug { revision, .. }
            | Request::SaveProfiles { revision, .. }
            | Request::ExportSettings { revision, .. }
            | Request::ImportSettings { revision, .. }
            | Request::Edit { revision, .. }
            | Request::Start { revision, .. }
            | Request::SaveMessenger { revision, .. } => revision,
            Request::Snapshot { .. }
            | Request::BaselineStatus {}
            | Request::Messenger {}
            | Request::TestMessenger { .. }
            | Request::PreviewSettings { .. } => {
                unreachable!()
            }
        };
        if revision != &self.revision(&saved)? {
            return Err("startup state changed since review; refresh before acting".into());
        }
        match request {
            Request::McpSave { settings, .. } => self.save_mcp(&mut saved, &settings),
            Request::McpDiscover { server, .. } => self.launch_mcp(&mut saved, Some(&server)),
            Request::McpPreview { .. } => self.launch_mcp(&mut saved, None),
            Request::McpApply {
                preview_id,
                selected,
                ..
            } => self.apply_mcp(&mut saved, &preview_id, &selected),
            Request::Save { config, .. } => {
                let mut config = *config;
                validate_profiles(&config.profiles)?;
                // DD-72: an empty workspace has no coordinator yet.
                let empty = config.profiles.is_empty() && config.coordinator.is_empty();
                if !empty && !config.profiles.iter().any(|p| p.id == config.coordinator) {
                    return Err("coordinator is not registered".into());
                }
                validate_name(&config.name)?;
                crate::roles::validate(&config.roles, &config.profiles)?;
                if config.goal.len() > 16 * 1024 {
                    return Err("goal exceeds 16 KiB".into());
                }
                if !config.repository.as_os_str().is_empty() {
                    config.repository = project::directory(&config.repository)?;
                    if self.broker.dir.starts_with(&config.repository) {
                        return Err("startup artifacts must be outside repository".into());
                    }
                }
                config.mcp = saved.config.mcp.clone();
                config.goal_sources = saved.config.goal_sources.clone();
                profiles::carry_over(&mut config.profiles, &saved.config.profiles);
                // DD-61: slugs are work records, changed only by slug operations.
                config.slugs = saved.config.slugs.clone();
                config.plan_rules = saved.config.plan_rules.clone();
                config.active_slug = saved.config.active_slug.clone();
                config.messenger = saved.config.messenger.clone();
                // DD-58: keep a reviewed plan unless what it was reviewed against changed.
                let keep = config.goal == saved.config.goal
                    && config.repository == saved.config.repository
                    && serde_json::to_value(config.workspace_mode).ok()
                        == serde_json::to_value(saved.config.workspace_mode).ok()
                    && profiles::execution_view(&config.profiles)?
                        == profiles::execution_view(&saved.config.profiles)?;
                let next = Saved {
                    imports: if config.goal == saved.config.goal {
                        saved.imports.clone()
                    } else {
                        None
                    },
                    mcp_result: if config.goal == saved.config.goal {
                        saved.mcp_result.clone()
                    } else {
                        None
                    },
                    config,
                    proposal: if keep { saved.proposal.clone() } else { None },
                    job: if keep { saved.job.clone() } else { None },
                    quota_job: saved.quota_job.clone(),
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
            Request::Quota { .. } => unreachable!("quota handled separately"),
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
            Request::Prepare { slug, .. } => self.start_planning(&mut saved, None, slug),
            Request::ReadPlan { path, slug, .. } => {
                self.start_planning(&mut saved, Some(&path), slug)
            }
            Request::NextPhase { .. } => self.next_phase(&mut saved),
            Request::SavePlanRules { scope, text, .. } => {
                self.save_plan_rules(&mut saved, &scope, text)
            }
            Request::CommitBaseline { .. } => self.commit_baseline(&saved),
            Request::IgnoreOsFiles { .. } => self.ignore_os_files(&saved),
            Request::CreateSlug { slug, .. } => self.create_slug(&mut saved, slug),
            Request::SaveProfiles {
                profiles, renames, ..
            } => self.save_profiles(&mut saved, profiles, &renames),
            Request::ExportSettings { folder, .. } => self.export_settings(&saved, &folder),
            Request::SaveMessenger {
                settings,
                url,
                token,
                relay_url,
                relay_secret,
                ..
            } => self.save_messenger(&mut saved, settings, [url, token, relay_url, relay_secret]),
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
                self.source_context(&saved)?;
                let repo = project::repository(&config.repository)?;
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
                    slug: None,
                    slug_phase: None,
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
                self.source_context(&saved)?;
                workflow::validate_permissions(&plan)?;
                let repo = Repository::open(&saved.config.repository)?;
                repo.clean()?;
                if repo.head()? != p.repository_base {
                    return Err("repository changed since planning; generate a new plan".into());
                }
                crate::state::Store::open(self.broker.dir.join("state.sqlite"))?.record("plan_execution_consent",&serde_json::json!({"digest":digest,"revision":revision,"repository_base":p.repository_base,"plan":path}),"Human approved the exact reviewed startup plan")?;
                let run = p.run.clone();
                let mut args = vec![
                    "startup-run".into(),
                    path.to_string_lossy().into(),
                    repo.root.to_string_lossy().into(),
                    run.to_string_lossy().into(),
                    digest,
                    p.repository_base.clone(),
                ];
                // DD-62: a phased proposal links its run to the slug phase.
                if let (Some(slug), Some(phase)) = (&p.slug, &p.slug_phase) {
                    let root = crate::slugs::root(&saved.config.repository, &self.broker.dir, slug);
                    args.extend([
                        "--phase".into(),
                        root.to_string_lossy().into(),
                        phase.clone(),
                    ]);
                }
                if saved.config.messenger.enabled {
                    crate::messenger::link_run(
                        &run,
                        &self.broker.dir,
                        &saved.config.name,
                        &saved.config.goal,
                        &saved.config.messenger,
                    )?;
                }
                saved.proposal.as_mut().unwrap().phase = "starting".into();
                saved.last_run = Some(run.clone());
                self.store(&saved)?;
                let mut result = self.launch(&mut saved, "start", args)?;
                result["run"] = serde_json::json!(run);
                Ok(result)
            }
            Request::Snapshot { .. }
            | Request::BaselineStatus {}
            | Request::Messenger {}
            | Request::TestMessenger { .. }
            | Request::PreviewSettings { .. } => {
                unreachable!()
            }
        }
    }
}
