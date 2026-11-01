use super::*;

impl Startup {
    pub(super) fn save_config(
        &self,
        saved: &mut Saved,
        config: Box<Config>,
    ) -> Result<serde_json::Value, String> {
        let mut config = *config;
        // DD-131: a changed executable drops full access before validation.
        super::profiles::derive_permissions(&mut config.profiles);
        validate_profiles(&config.profiles)?;
        // DD-72: an empty workspace has no coordinator yet.
        let empty = config.profiles.is_empty() && config.coordinator.is_empty();
        if !empty && !config.profiles.iter().any(|p| p.id == config.coordinator) {
            return Err(crate::ui::messages::text("native.coordinator_is_not_registered").into());
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
        config.active_slug = (config.repository == saved.config.repository)
            .then(|| saved.config.active_slug.clone())
            .flatten();
        config.messenger = saved.config.messenger.clone();
        // DD-58: keep a reviewed plan unless what it was reviewed against changed.
        let keep = config.goal == saved.config.goal
            && config.repository == saved.config.repository
            && serde_json::to_value(config.workspace_mode).ok()
                == serde_json::to_value(saved.config.workspace_mode).ok()
            && profiles::execution_view(&config.profiles)?
                == profiles::execution_view(&saved.config.profiles)?;
        let keep_optimizer = config.repository == saved.config.repository
            && saved.optimize_goal.as_deref() == Some(config.goal.as_str());
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
            optimize_result: keep_optimizer
                .then(|| saved.optimize_result.clone())
                .flatten(),
            optimize_goal: keep_optimizer
                .then(|| saved.optimize_goal.clone())
                .flatten(),
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
}
