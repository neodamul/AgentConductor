use super::*;

impl Startup {
    pub(super) fn dispatch_plan(
        &self,
        saved: &mut Saved,
        request: Request,
    ) -> Result<serde_json::Value, String> {
        match request {
            // DD-45: a hand-written plan starts from an empty form; no coordinator or job.
            Request::Draft { .. } => {
                let config = &saved.config;
                if config.goal.trim().is_empty() {
                    return Err(crate::ui::messages::text("native.enter_a_nonempty_goal").into());
                }
                validate_profiles(&config.profiles)?;
                let agent = &config
                    .profiles
                    .first()
                    .ok_or(crate::ui::messages::text(
                        "native.register_an_agent_before_writing_a_plan",
                    ))?
                    .id;
                self.source_context(saved)?;
                let repo = self.execution_repository(saved)?;
                repo.clean()?;
                if self.broker.dir.starts_with(&repo.root) {
                    return Err(crate::ui::messages::text(
                        "native.startup_artifacts_must_be_outside_repository",
                    )
                    .into());
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
                            execution_options: Default::default(),
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
                    workspace: Some(repo.root.clone()),
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
                self.store(saved)?;
                Ok(serde_json::json!({"drafted":true}))
            }
            Request::Edit {
                tasks,
                workspace_mode,
                digest,
                ..
            } => {
                let p = saved
                    .proposal
                    .as_ref()
                    .ok_or(crate::ui::messages::text("native.no_generated_plan"))?;
                if p.phase != "ready" {
                    return Err(
                        crate::ui::messages::text("native.plan_is_not_ready_for_review").into(),
                    );
                }
                if let Some(expected) = digest {
                    let current: Plan = read(&p.directory.join("review.json"))?;
                    let actual = workflow::digest(
                        &serde_json::to_vec(&current).map_err(|error| error.to_string())?,
                    );
                    if expected != actual {
                        return Err("the reviewed plan changed; refresh before saving edits".into());
                    }
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
                self.store(saved)?;
                Ok(serde_json::json!({"saved":true}))
            }
            Request::Start { digest, revision } => {
                let p = saved
                    .proposal
                    .as_ref()
                    .ok_or(crate::ui::messages::text("native.no_generated_plan"))?;
                if p.phase != "ready" {
                    return Err(crate::ui::messages::text(
                        "native.plan_is_not_ready_for_execution_consent",
                    )
                    .into());
                }
                let path = p.directory.join("review.json");
                let plan: Plan = read(&path)?;
                plan.validate()?;
                // Goal folders (<project>/<slug>/) hold plans and progress, never task work.
                if !saved.config.repository.as_os_str().is_empty()
                    && let Some(path) = plan.tasks.iter().flat_map(|t| &t.paths).find(|p| {
                        p.split('/').next().is_some_and(|first| {
                            saved
                                .config
                                .slugs
                                .iter()
                                .any(|s| first.eq_ignore_ascii_case(&s.slug))
                        })
                    })
                {
                    return Err(crate::ui::messages::render(
                        "native.task_scope_is_inside_a_goal_folder",
                        &[("path", &path.to_string())],
                    ));
                }
                // Executables, arguments and permissions must match; catalogs may be newer.
                if plan.goal != saved.config.goal
                    || profiles::execution_view(&plan.agents)?
                        != profiles::execution_view(&saved.config.profiles)?
                {
                    return Err(crate::ui::messages::text(
                        "native.reviewed_plan_changed_the_registered_profiles_or_goal",
                    )
                    .into());
                }
                if workflow::digest(&serde_json::to_vec(&plan).map_err(|e| e.to_string())?)
                    != digest
                {
                    return Err(crate::ui::messages::text(
                        "native.approve_the_exact_reviewed_plan_digest",
                    )
                    .into());
                }
                self.source_context(saved)?;
                workflow::validate_permissions(&plan)?;
                let repo =
                    Repository::open(p.workspace.as_deref().unwrap_or(&saved.config.repository))?;
                if let Some(source) = crate::folder::load(&repo.root)? {
                    source.validate()?;
                }
                repo.clean()?;
                if repo.head()? != p.repository_base {
                    return Err(crate::ui::messages::text(
                        "native.repository_changed_since_planning_generate_a_new_plan",
                    )
                    .into());
                }
                crate::state::Store::open(self.broker.dir.join("state.sqlite"))?.record("plan_execution_consent",&serde_json::json!({"digest":digest,"revision":revision,"repository_base":p.repository_base,"plan":path}),crate::ui::messages::text("native.human_approved_the_exact_reviewed_startup_plan"))?;
                let repo = crate::folder::activate(&repo, &saved.config.repository)?;
                let run = p.run.clone();
                workflow::set_auto_resume(&run, saved.config.auto_resume)?;
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
                    let root = crate::slugs::root(
                        &saved.config.repository,
                        &self.broker.dir,
                        &saved.config.name,
                        slug,
                    );
                    workflow::link_phase_at_launch(&run, &root, phase)?;
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
                // The coordinator orchestrates the run, including tool requests that the
                // workspace sandbox cannot permit by itself.
                if let Some(coordinator) = saved
                    .config
                    .profiles
                    .iter()
                    .find(|p| p.id == saved.config.coordinator)
                {
                    std::fs::write(
                        crate::native_guard::orchestrator_path(&run),
                        serde_json::to_vec_pretty(coordinator).map_err(|e| e.to_string())?,
                    )
                    .map_err(|e| e.to_string())?;
                }
                saved.proposal.as_mut().unwrap().phase = "starting".into();
                saved.last_run = Some(run.clone());
                self.store(saved)?;
                let mut result = self.launch(saved, "start", args)?;
                result["run"] = serde_json::json!(run);
                Ok(result)
            }
            _ => unreachable!(),
        }
    }
}
