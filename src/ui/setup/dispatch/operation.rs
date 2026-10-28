use super::*;

impl Startup {
    pub(super) fn dispatch_operation(
        &self,
        request: Request,
        saved: &mut Saved,
    ) -> Result<serde_json::Value, String> {
        match request {
            Request::StopPlanning {}
            | Request::CachePreview {}
            | Request::Chat { .. }
            | Request::ChatHistory {}
            | Request::ChatReset {}
            | Request::CacheClear { .. } => unreachable!(),
            Request::McpSave { settings, .. } => self.save_mcp(saved, &settings),
            Request::McpDiscover { server, .. } => self.launch_mcp(saved, Some(&server)),
            Request::McpPreview { .. } => self.launch_mcp(saved, None),
            Request::OptimizeGoal { goal, .. } => self.launch_optimize(saved, goal),
            Request::McpApply {
                preview_id,
                selected,
                ..
            } => self.apply_mcp(saved, &preview_id, &selected),
            Request::Save { config, .. } => {
                let mut config = *config;
                // DD-131: a changed executable drops full access before validation.
                super::super::profiles::derive_permissions(&mut config.profiles);
                validate_profiles(&config.profiles)?;
                // DD-72: an empty workspace has no coordinator yet.
                let empty = config.profiles.is_empty() && config.coordinator.is_empty();
                if !empty && !config.profiles.iter().any(|p| p.id == config.coordinator) {
                    return Err(
                        crate::ui::messages::text("native.coordinator_is_not_registered").into(),
                    );
                }
                validate_name(&config.name)?;
                crate::roles::validate(&config.roles, &config.profiles)?;
                if config.goal.len() > 16 * 1024 {
                    return Err(crate::ui::messages::text("native.goal_exceeds_16_kib").into());
                }
                if !config.repository.as_os_str().is_empty() {
                    config.repository = project::directory(&config.repository)?;
                    if self.broker.dir.starts_with(&config.repository) {
                        return Err(crate::ui::messages::text(
                            "native.startup_artifacts_must_be_outside_repository",
                        )
                        .into());
                    }
                }
                config.mcp = saved.config.mcp.clone();
                config.goal_sources = saved.config.goal_sources.clone();
                profiles::carry_over(&mut config.profiles, &saved.config.profiles);
                config
                    .profiles
                    .iter_mut()
                    .for_each(profiles::derive_permission);
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
                    chat_result: saved.chat_result.clone(),
                    optimize_result: saved.optimize_result.clone(),
                    config,
                    proposal: if keep { saved.proposal.clone() } else { None },
                    job: if keep { saved.job.clone() } else { None },
                    quota_job: saved.quota_job.clone(),
                    metadata_queue: saved.metadata_queue.clone(),
                    metadata_input: saved.metadata_input.clone(),
                    metadata_settled: saved.metadata_settled.clone(),
                    metadata_error: saved.metadata_error.clone(),
                    last_run: saved.last_run.clone(),
                };
                self.store(&next)?;
                *saved = next;
                Ok(serde_json::json!({"saved":true}))
            }
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
            } => self.edit_plan(saved, tasks, workspace_mode, digest),
            Request::Start { digest, revision } => self.start_plan(saved, digest, revision),
            Request::Snapshot { .. }
            | Request::BaselineStatus {}
            | Request::Messenger {}
            | Request::TestMessenger { .. }
            | Request::PreviewSettings { .. } => unreachable!(),
        }
    }
}
