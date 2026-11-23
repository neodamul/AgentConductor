//! #997: reject completed-Goal writes before bookkeeping or child-process launch.
use super::*;

impl Startup {
    pub(super) fn guard_completed_request(
        &self,
        saved: &Saved,
        request: &Request,
    ) -> Result<(), String> {
        let mutation = matches!(
            request,
            Request::Save { .. }
                | Request::SaveAutoMerge { .. }
                | Request::Prepare { .. }
                | Request::Draft { .. }
                | Request::ReadPlan { .. }
                | Request::Edit { .. }
                | Request::Start { .. }
                | Request::ImportSettings { .. }
                | Request::OptimizeGoal { .. }
                | Request::McpApply { .. }
                | Request::McpPreview { .. }
                | Request::CreateSlug { .. }
                | Request::CommitBaseline { .. }
                | Request::IgnoreOsFiles { .. }
        );
        if !mutation {
            return Ok(());
        }
        // GitHub #1085: an active Goal with phase progress is complete only when its progress says so; the last
        // run report is just the finished Phase of a multi-phase Goal.
        let goals = self.slug_snapshot(saved);
        let stage = goals.as_array().and_then(|goals| {
            goals
                .iter()
                .find(|goal| goal["active"] == true)
                .and_then(|goal| goal["progress"]["stage"].as_str().map(str::to_owned))
        });
        // Only a phased plan has authoritative progress: `phase` is mid-Goal, `completed` is done; earlier
        // stages (no_plan, analysis) say nothing about a recorded run.
        match stage.as_deref() {
            Some("completed") => return Err(messages::text("goal.readonly_reason").into()),
            Some("phase") => return Ok(()),
            _ => {}
        }
        let run = self.last_run(saved).or_else(|| {
            saved
                .proposal
                .as_ref()
                .map(|p| p.run.clone())
                .filter(|p| p.join("report.json").is_file())
        });
        if let Some(run) = run {
            workflow::ensure_goal_mutable(&workflow::load(&run)?)?;
        }
        Ok(())
    }
}
