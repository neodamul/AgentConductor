use super::*;
impl Plan {
    pub fn validate(&self) -> Result<(), String> {
        // DD-60: one to twelve tasks (R-53 supersedes the original fixed three).
        if self.goal.trim().is_empty() || !(1..=12).contains(&self.tasks.len()) {
            return Err("a nonempty goal and one to twelve tasks are required".into());
        }
        let mut agents = HashSet::new();
        for agent in &self.agents {
            agent.validate_custom_configuration()?;
            if agent.parser != crate::protocol::Provider::Custom
                && (!agent.event_types.is_empty() || agent.session_id_pointer.is_some())
            {
                return Err("custom event mappings require a custom CLI profile".into());
            }
            if agent
                .event_types
                .iter()
                .any(|(name, _)| name.is_empty() || name.chars().any(char::is_control))
                || agent
                    .session_id_pointer
                    .as_ref()
                    .is_some_and(|pointer| !pointer.is_empty() && !pointer.starts_with('/'))
            {
                return Err("invalid custom event mapping or session JSON Pointer".into());
            }
            if let Some(quota) = &agent.quota_command
                && (agent.parser != crate::protocol::Provider::Custom
                    || quota.args.len() > 64
                    || !(1..=300).contains(&quota.timeout_seconds)
                    || quota
                        .args
                        .iter()
                        .any(|arg| arg.len() > 4096 || arg.contains('\0'))
                    || quota
                        .output_pointer
                        .as_ref()
                        .is_some_and(|pointer| !pointer.is_empty() && !pointer.starts_with('/')))
            {
                return Err("invalid custom quota command configuration".into());
            }
            if agent
                .environment
                .keys()
                .any(|key| key.starts_with("AC_CHECKPOINT_"))
            {
                return Err("checkpoint bootstrap environment is coordinator-owned".into());
            }
            if agent.cooperative == CooperativeMode::CheckpointV1
                && (agent.parser != crate::protocol::Provider::Custom
                    || agent
                        .args
                        .iter()
                        .map(|arg| arg.matches("{session_id}").count())
                        .sum::<usize>()
                        != 1)
            {
                return Err(
                    "checkpoint_v1 requires a custom CLI with one {session_id} argv binding".into(),
                );
            }
            let executable = crate::executable::program_name(&agent.executable);
            if agent.id.trim().is_empty()
                || !agents.insert(&agent.id)
                || executable.is_empty()
                || !(1..=3).contains(&agent.max_sessions)
                || !(1..=60).contains(&agent.quota_refresh_minutes)
                || matches!(
                    executable.as_str(),
                    "sh" | "bash" | "zsh" | "cmd" | "powershell" | "pwsh"
                )
            {
                return Err("invalid or duplicate agent profile".into());
            }
        }
        // Agents may name each other: execution stops at the first agent already tried,
        // so a chain where every account is limited waits for availability.
        for current in &self.agents {
            let Some(fallback) = &current.fallback else {
                continue;
            };
            let target = self
                .agents
                .iter()
                .find(|agent| agent.id == fallback.agent)
                .ok_or("unknown fallback agent")?;
            if current.quota_key() == target.quota_key() {
                return Err("fallback must use a different account quota group".into());
            }
            if let Some(model) = &fallback.model {
                target.validate_model(model)?;
            }
        }
        let mut ids = HashSet::new();
        let mut scopes: Vec<&str> = vec![];
        for task in &self.tasks {
            // GitHub #251: name the task and the field, so a coordinator (or a person) can correct exactly that.
            let id = &task.id;
            if id.trim().is_empty() {
                return Err("task id is empty".into());
            }
            if !ids.insert(&task.id) {
                return Err(format!("task {id}: duplicate task id"));
            }
            if !agents.contains(&task.agent) {
                return Err(format!("task {id}: unknown agent {}", task.agent));
            }
            if task.prompt.trim().is_empty() {
                return Err(format!("task {id}: prompt is empty"));
            }
            if task.paths.is_empty() {
                return Err(format!("task {id}: {EMPTY_PATHS}"));
            }
            if task
                .test_command
                .first()
                .is_none_or(|executable| executable.is_empty())
            {
                return Err(format!(
                    "task {id}: test_command must be an executable plus argv, and the executable is missing"
                ));
            }
            task.fallback_for(&self.agents)?;
            if let Some(model) = &task.model {
                self.agents
                    .iter()
                    .find(|a| a.id == task.agent)
                    .unwrap()
                    .validate_model(model)?;
            }
            let agent = self
                .agents
                .iter()
                .find(|agent| agent.id == task.agent)
                .unwrap();
            for (name, value) in &task.execution_options {
                let option = agent
                    .execution_options
                    .iter()
                    .find(|option| option.name == *name)
                    .ok_or_else(|| {
                        format!("task {} selects unknown option group {name}", task.id)
                    })?;
                let model = task
                    .execution_options
                    .get("model")
                    .map(String::as_str)
                    .or(task.model.as_deref())
                    .or_else(|| {
                        agent
                            .execution_options
                            .iter()
                            .find(|entry| entry.name == "model")
                            .and_then(|entry| entry.default.as_deref())
                    });
                let allowed = if name == "model" {
                    agent.validate_model(value).map(|_| true)
                } else {
                    option
                        .allowed_values(model)
                        .map(|allowed| allowed.contains(value))
                        .ok_or_else(|| {
                            format!("task {} uses {name} unavailable for its model", task.id)
                        })
                }?;
                if !allowed {
                    return Err(format!(
                        "task {} selects invalid {name} value for its model",
                        task.id
                    ));
                }
            }
            for option in &agent.execution_options {
                if option.name == "model" || matches!(option.name.as_str(), "resume" | "session") {
                    continue;
                }
                if task.execution_options.contains_key(&option.name) {
                    continue;
                }
                let Some(value) = option.default.as_deref() else {
                    continue;
                };
                let model = task
                    .execution_options
                    .get("model")
                    .map(String::as_str)
                    .or(task.model.as_deref())
                    .or_else(|| {
                        agent
                            .execution_options
                            .iter()
                            .find(|entry| entry.name == "model")
                            .and_then(|entry| entry.default.as_deref())
                    });
                if !option
                    .allowed_values(model)
                    .is_some_and(|allowed| allowed.iter().any(|allowed| allowed == value))
                {
                    return Err(format!(
                        "task {} has invalid default for {} on its model",
                        task.id, option.name
                    ));
                }
            }
            for path in &task.paths {
                if path.is_empty()
                    || path.contains(['\\', '*', '?', '[', ']'])
                    || path.split('/').any(|part| {
                        matches!(
                            part.to_ascii_lowercase().as_str(),
                            "" | "." | ".." | ".git" | ".agentconductor"
                        )
                    })
                    || scopes.iter().any(|other| overlaps(path, other))
                {
                    return Err(format!("unsafe or overlapping scope: {path}"));
                }
                scopes.push(path);
            }
        }
        let mut done = HashSet::new();
        while done.len() < self.tasks.len() {
            let previous = done.len();
            for task in &self.tasks {
                if let Some(id) = task.depends_on.iter().find(|id| !ids.contains(id)) {
                    return Err(format!("task {} has unknown dependency {id}", task.id));
                }
                if task.depends_on.iter().all(|id| done.contains(id)) {
                    done.insert(&task.id);
                }
            }
            if done.len() == previous {
                return Err("cyclic task dependencies".into());
            }
        }
        Ok(())
    }
}
