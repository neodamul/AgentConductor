use super::*;

#[path = "draft.rs"]
mod draft;
#[path = "save.rs"]
mod save;

impl Startup {
    pub(super) fn dispatch_operation(
        &self,
        request: Request,
        saved: &mut Saved,
    ) -> Result<serde_json::Value, String> {
        match request {
            Request::StopPlanning {} | Request::CachePreview {} | Request::CacheClear { .. } => {
                unreachable!()
            }
            Request::McpSave { settings, .. } => self.save_mcp(saved, &settings),
            Request::McpDiscover { server, .. } => self.launch_mcp(saved, Some(&server)),
            Request::McpPreview { .. } => self.launch_mcp(saved, None),
            Request::McpApply {
                preview_id,
                selected,
                ..
            } => self.apply_mcp(saved, &preview_id, &selected),
            Request::Save { config, .. } => self.save_config(saved, config),
            Request::Catalog {
                agent, mut catalog, ..
            } => {
                validate_catalog(&catalog)?;
                catalog.origin = crate::plan::CatalogOrigin::Manual;
                let mut next = saved.clone();
                next.config
                    .profiles
                    .iter_mut()
                    .find(|p| p.id == agent)
                    .ok_or(crate::ui::messages::text("native.unknown_agent"))?
                    .catalog = Some(catalog);
                // DD-57: observed models are metadata; a reviewed plan stays valid.
                next.job = None;
                self.store(&next)?;
                *saved = next;
                Ok(serde_json::json!({"saved":true}))
            }
            Request::Quota { .. } => unreachable!("quota handled separately"),
            Request::Discover { .. } => unreachable!("metadata handled separately"),
            Request::Prepare { slug, .. } => self.start_planning(saved, None, slug),
            Request::ReadPlan { path, slug, .. } => self.start_planning(saved, Some(&path), slug),
            Request::NextPhase { .. } => self.next_phase(saved),
            Request::SavePlanRules { scope, text, .. } => self.save_plan_rules(saved, &scope, text),
            Request::CommitBaseline { .. } => self.commit_baseline(saved),
            Request::IgnoreOsFiles { .. } => self.ignore_os_files(saved),
            Request::NewGoal { .. } => self.change_goal(saved, None),
            Request::SelectGoal { slug, .. } => self.change_goal(saved, Some(slug)),
            Request::CreateSlug { slug, .. } => self.create_slug(saved, slug),
            Request::SaveProfiles {
                profiles, renames, ..
            } => self.save_profiles(saved, profiles, &renames),
            Request::ExportSettings { folder, .. } => self.export_settings(saved, &folder),
            Request::SaveMessenger {
                settings,
                url,
                token,
                relay_url,
                relay_secret,
                ..
            } => self.save_messenger(saved, settings, [url, token, relay_url, relay_secret]),
            Request::ImportSettings {
                folder,
                keep_verified,
                ..
            } => self.import_settings(saved, &folder, keep_verified),
            // DD-45: a hand-written plan starts from an empty form; no coordinator or job.
            Request::Draft { .. } => self.draft_plan(saved),
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
