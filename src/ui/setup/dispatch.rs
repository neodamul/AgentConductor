use super::*;

mod complex;

impl Startup {
    pub fn dispatch(&self, bytes: &[u8]) -> Result<serde_json::Value, String> {
        if bytes.len() > 64 * 1024 {
            return Err(crate::ui::messages::text("native.native_request_exceeds_64_kib").into());
        }
        let request: Request = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        let mut saved = self
            .saved
            .lock()
            .map_err(|_| crate::ui::messages::text("native.startup_settings_poisoned"))?;
        if matches!(request, Request::StopPlanning {}) {
            return self.stop_planning(&mut saved);
        }
        if matches!(request, Request::CachePreview {}) {
            return self.cache_preview(&saved);
        }
        if let Request::CacheClear { digest } = &request {
            return self.clear_cache(&saved, Some(digest), 0);
        }
        let metadata_job = self.metadata_tick(&mut saved)?;
        let job = self.job(&saved)?;
        self.settle(&mut saved, &job)?;
        if let Request::Snapshot {
            icons,
            mcp_result,
            quota_refresh_last,
        } = request
        {
            return self.snapshot_state(
                &saved,
                &job,
                &metadata_job,
                icons,
                mcp_result,
                &quota_refresh_last,
            );
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
        if let Request::Quota { agent, revision } | Request::Discover { agent, revision } = &request
        {
            if revision != &self.revision(&saved)? {
                return Err(crate::ui::messages::text(
                    "native.startup_state_changed_since_review_refresh_before_acting",
                )
                .into());
            }
            let kind = if matches!(request, Request::Quota { .. }) {
                "quota"
            } else {
                "catalog"
            };
            return self.enqueue_metadata(&mut saved, kind, agent);
        }
        if job["state"] == "running" {
            return Err(crate::ui::messages::text("native.startup_operation_still_running").into());
        }
        self.ensure_no_live_jobs()?;
        if matches!(
            request,
            Request::NewGoal { .. }
                | Request::Prepare { .. }
                | Request::Draft { .. }
                | Request::ReadPlan { .. }
                | Request::NextPhase { .. }
                | Request::Start { .. }
        ) {
            crate::ui::workspaces::project::validate_owner(
                &saved.config.repository,
                &self.broker.dir,
            )?;
        }
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
            | Request::NewGoal { revision }
            | Request::SelectGoal { revision, .. }
            | Request::CreateSlug { revision, .. }
            | Request::SaveProfiles { revision, .. }
            | Request::ExportSettings { revision, .. }
            | Request::ImportSettings { revision, .. }
            | Request::Edit { revision, .. }
            | Request::Start { revision, .. }
            | Request::SaveMessenger { revision, .. } => revision,
            Request::CachePreview {}
            | Request::CacheClear { .. }
            | Request::StopPlanning {}
            | Request::Snapshot { .. }
            | Request::BaselineStatus {}
            | Request::Messenger {}
            | Request::TestMessenger { .. }
            | Request::PreviewSettings { .. } => {
                unreachable!()
            }
        };
        if revision != &self.revision(&saved)? {
            return Err(crate::ui::messages::text(
                "native.startup_state_changed_since_review_refresh_before_acting",
            )
            .into());
        }
        match request {
            Request::StopPlanning {} | Request::CachePreview {} | Request::CacheClear { .. } => {
                unreachable!()
            }
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
                // DD-131: a changed executable drops full access before validation.
                super::profiles::derive_permissions(&mut config.profiles);
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
            Request::NewGoal { .. } => self.change_goal(&mut saved, None),
            Request::SelectGoal { slug, .. } => self.change_goal(&mut saved, Some(slug)),
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
            Request::Draft { .. } => self.draft_plan(&mut saved),
            Request::Edit {
                tasks,
                workspace_mode,
                digest,
                ..
            } => self.edit_plan(&mut saved, tasks, workspace_mode, digest),
            Request::Start { digest, revision } => self.start_plan(&mut saved, digest, revision),
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
