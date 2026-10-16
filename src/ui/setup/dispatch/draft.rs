use super::*;

impl Startup {
    pub(super) fn draft_plan(&self, saved: &mut Saved) -> Result<serde_json::Value, String> {
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
}
