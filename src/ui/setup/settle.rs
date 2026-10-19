use super::*;
use crate::plan::{AgentProfile, Plan};

impl Startup {
    pub(super) fn settle(&self, s: &mut Saved, job: &serde_json::Value) -> Result<(), String> {
        if job["state"] != "completed" {
            return Ok(());
        }
        if s.job.as_ref().is_some_and(|j| j.operation == "plan")
            && s.proposal.as_ref().is_some_and(|p| p.phase == "planning")
        {
            let p = s.proposal.as_mut().unwrap();
            let mut plan: Plan = read(&p.directory.join("plan.json"))?;
            if plan.goal != s.config.goal
                || profiles::execution_view(&plan.agents)?
                    != profiles::execution_view(&s.config.profiles)?
            {
                return Err(
                    messages::text("native.planner_changed_registered_profiles_or_goal").into(),
                );
            }
            plan.workspace_mode = s.config.workspace_mode;
            plan.validate()?;
            write(&p.directory.join("review.json"), &plan)?;
            p.phase = "ready".into();
            crate::messenger::notify_plan(
                &self.broker.dir,
                &s.config.messenger,
                &s.config.name,
                &s.config.goal,
                plan.tasks.len(),
            );
            slug::adopt_planned_slug(&mut s.config, &self.broker.dir, p)?;
            if let Some(slug) = &p.slug {
                let root = crate::slugs::root(
                    &s.config.repository,
                    &self.broker.dir,
                    &s.config.name,
                    slug,
                );
                p.slug_phase = crate::slug_progress::progress(&root)["current_phase"]
                    .as_str()
                    .map(str::to_owned);
            }
            self.store(s)?;
        } else if s
            .job
            .as_ref()
            .is_some_and(|j| j.operation.starts_with("catalog:"))
        {
            let profiles: Vec<AgentProfile> = read(&self.broker.dir.join("profiles.json"))?;
            validate_profiles(&profiles)?;
            let mut expected = s.config.profiles.clone();
            for (a, b) in expected.iter_mut().zip(&profiles) {
                a.catalog = b.catalog.clone();
            }
            if serde_json::to_vec(&expected).map_err(|e| e.to_string())?
                != serde_json::to_vec(&profiles).map_err(|e| e.to_string())?
            {
                return Err(messages::text(
                    "native.discovery_changed_registered_execution_settings",
                )
                .into());
            }
            s.config.profiles = profiles;
            s.job = None;
            self.store(s)?;
        }
        Ok(())
    }
}
