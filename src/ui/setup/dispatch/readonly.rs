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
        if self.slug_snapshot(saved).as_array().is_some_and(|goals| {
            goals
                .iter()
                .any(|goal| goal["active"] == true && goal["progress"]["stage"] == "completed")
        }) {
            return Err(messages::text("goal.readonly_reason").into());
        }
        Ok(())
    }
}
