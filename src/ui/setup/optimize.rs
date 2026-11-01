use super::*;

impl Startup {
    pub(super) fn launch_optimize(
        &self,
        saved: &mut Saved,
        goal: String,
    ) -> Result<serde_json::Value, String> {
        crate::prompt_optimizer::build_prompt(&goal, "English")?;
        crate::mcp::coordinator_profile(&saved.config.profiles, &saved.config.coordinator)?;
        if !saved.config.repository.is_dir() {
            return Err(crate::ui::messages::text("native.choose_a_project_folder").into());
        }
        let id = crate::native_guard::session_id()?;
        let language = if crate::ui::language::load() == "ko" {
            "Korean"
        } else {
            "English"
        };
        let input_name = format!("optimize-input-{id}.json");
        crate::mcp::store(
            &self.broker.dir,
            &input_name,
            &serde_json::to_vec(&serde_json::json!({"goal": goal.trim(), "language": language}))
                .map_err(|error| error.to_string())?,
        )?;
        let previous = (saved.optimize_result.clone(), saved.optimize_goal.clone());
        saved.optimize_result = Some(format!("optimize-{id}.json"));
        saved.optimize_goal = Some(goal.trim().to_owned());
        match self.launch(
            saved,
            "optimize",
            vec![
                "optimize".into(),
                self.broker.dir.to_string_lossy().into(),
                id,
            ],
        ) {
            Ok(result) => {
                if let Some(name) = previous.0.as_deref() {
                    remove_optimizer_artifact(&self.broker.dir, name);
                }
                Ok(result)
            }
            Err(error) => {
                saved.optimize_result = previous.0;
                saved.optimize_goal = previous.1;
                let _ = fs::remove_file(self.broker.dir.join("mcp").join(input_name));
                Err(error)
            }
        }
    }

    pub(super) fn optimize_snapshot(
        &self,
        saved: &Saved,
        job: &serde_json::Value,
    ) -> serde_json::Value {
        let Some(id) = job["id"].as_str() else {
            return serde_json::Value::Null;
        };
        if job["kind"] != "optimize" || job["state"] != "completed" {
            return serde_json::Value::Null;
        }
        let name = format!("optimize-{id}.json");
        if saved.optimize_result.as_deref() != Some(name.as_str()) {
            return serde_json::Value::Null;
        }
        let path = self.broker.dir.join("mcp").join(name);
        let proposal = crate::mcp::load(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
        proposal.map_or(
            serde_json::Value::Null,
            |proposal| serde_json::json!({"goal": saved.optimize_goal, "proposal": proposal}),
        )
    }
}

fn remove_optimizer_artifact(root: &Path, name: &str) {
    if name.starts_with("optimize-")
        && name.ends_with(".json")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.')
    {
        let _ = fs::remove_file(root.join("mcp").join(name));
    }
}
