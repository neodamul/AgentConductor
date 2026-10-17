use super::*;

mod operation;

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
        self.dispatch_operation(request, &mut saved)
    }
}
