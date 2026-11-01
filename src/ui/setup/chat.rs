//! Bounded coordinator chat jobs, using the configured CLI and its normal login.
use super::*;

impl Startup {
    pub(super) fn launch_chat(
        &self,
        saved: &mut Saved,
        question: String,
        revision: &str,
    ) -> Result<serde_json::Value, String> {
        crate::chat::validate_question(&question)?;
        let mut turns = self.stored_chat_turns(saved)?;
        turns.push(crate::chat::Turn {
            role: "operator".into(),
            text: question,
        });
        let turns = crate::chat::recent_history(&turns);
        let language = if crate::ui::language::load() == "ko" {
            "Korean"
        } else {
            "English"
        };
        let saved_value = serde_json::to_value(&*saved).map_err(|error| error.to_string())?;
        let workspace = crate::chat_context::workspace(&saved_value, &saved.config.repository);
        crate::chat::build_prompt_in(&turns, language, &workspace)?;
        crate::mcp::coordinator_profile(&saved.config.profiles, &saved.config.coordinator)?;
        let id = crate::native_guard::session_id()?;
        crate::mcp::store(
            &self.broker.dir,
            &format!("chat-input-{id}.json"),
            &serde_json::to_vec(&serde_json::json!({
                "turns": turns,
                "language": language,
                "workspace": workspace
            }))
            .map_err(|e| e.to_string())?,
        )?;
        let old_result = saved.chat_result.clone();
        saved.chat_result = Some(format!("chat-{id}.json"));
        let result = self.launch_with_revision(
            saved,
            "chat",
            vec![
                "chat".into(),
                self.broker.dir.to_string_lossy().into(),
                id.clone(),
            ],
            revision,
        );
        if result.is_err() {
            saved.chat_result = old_result;
        } else {
            let _ = clear_previous_chat_artifacts(&self.broker.dir, Some(&id));
        }
        result
    }

    pub(super) fn dispatch_chat(
        &self,
        saved: &mut Saved,
        revision: &str,
        text: String,
    ) -> Result<serde_json::Value, String> {
        if revision != self.chat_revision(saved)? {
            return Err(crate::ui::messages::text(
                "native.startup_state_changed_since_review_refresh_before_acting",
            )
            .into());
        }
        let job = self.job(saved)?;
        self.settle(saved, &job)?;
        if job["state"] == "running" {
            return Err(crate::ui::messages::text("native.startup_operation_still_running").into());
        }
        self.ensure_no_live_jobs()?;
        self.launch_chat(saved, text, revision)
    }

    pub(super) fn chat_history(
        &self,
        saved: &Saved,
        _job: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        serde_json::to_value(self.stored_chat_turns(saved)?).map_err(|e| e.to_string())
    }

    pub(super) fn chat_snapshot(
        &self,
        saved: &Saved,
        job: &serde_json::Value,
    ) -> serde_json::Value {
        if job["kind"] != "chat" || job["state"] != "completed" {
            return serde_json::Value::Null;
        }
        let Some(name) = &saved.chat_result else {
            return serde_json::Value::Null;
        };
        let answer = self.chat_answer(name);
        let Some(answer) = answer else {
            return serde_json::Value::Null;
        };
        serde_json::json!({"job":job["id"],"answer":answer})
    }

    pub(super) fn reset_chat(&self, saved: &mut Saved) -> Result<serde_json::Value, String> {
        saved.chat_result = None;
        self.store(saved)?;
        let _ = clear_previous_chat_artifacts(&self.broker.dir, None);
        Ok(serde_json::json!({"reset":true}))
    }

    fn chat_answer(&self, name: &str) -> Option<String> {
        let id = chat_id(name)?;
        let path = self.broker.dir.join("mcp").join(format!("chat-{id}.json"));
        crate::mcp::load(&path)
            .ok()
            .and_then(|data| serde_json::from_slice::<serde_json::Value>(&data).ok())
            .and_then(|value| value["answer"].as_str().map(str::to_owned))
    }

    fn stored_chat_turns(&self, saved: &Saved) -> Result<Vec<crate::chat::Turn>, String> {
        let Some(name) = &saved.chat_result else {
            return Ok(Vec::new());
        };
        let id = chat_id(name).ok_or("stored coordinator chat reference is invalid")?;
        let input = self
            .broker
            .dir
            .join("mcp")
            .join(format!("chat-input-{id}.json"));
        let mut turns = if input.exists() {
            let bytes = crate::mcp::load(&input).map_err(|error| error.to_string())?;
            let value: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            serde_json::from_value(value["turns"].clone()).map_err(|e| e.to_string())?
        } else {
            Vec::new()
        };
        if let Some(answer) = self.chat_answer(name) {
            turns.push(crate::chat::Turn {
                role: "assistant".into(),
                text: answer,
            });
        }
        Ok(crate::chat::recent_history(&turns))
    }
}

fn chat_id(name: &str) -> Option<&str> {
    let id = name.strip_prefix("chat-")?.strip_suffix(".json")?;
    (!id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'-'))
    .then_some(id)
}

fn clear_previous_chat_artifacts(root: &Path, keep: Option<&str>) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry.file_name().to_string_lossy().starts_with("chat-run-")
            && !keep
                .is_some_and(|id| entry.file_name().to_string_lossy() == format!("chat-run-{id}"))
            && entry
                .file_type()
                .map_err(|error| error.to_string())?
                .is_dir()
        {
            fs::remove_dir_all(entry.path()).map_err(|error| error.to_string())?;
        }
    }
    let artifacts = root.join("mcp");
    if let Ok(entries) = fs::read_dir(artifacts) {
        for entry in entries {
            let entry = entry.map_err(|error| error.to_string())?;
            let name = entry.file_name();
            if name.to_string_lossy().starts_with("chat-")
                && name.to_string_lossy().ends_with(".json")
                && !keep.is_some_and(|id| {
                    name.to_string_lossy() == format!("chat-{id}.json")
                        || name.to_string_lossy() == format!("chat-input-{id}.json")
                })
                && entry
                    .file_type()
                    .map_err(|error| error.to_string())?
                    .is_file()
            {
                fs::remove_file(entry.path()).map_err(|error| error.to_string())?;
            }
        }
    }
    Ok(())
}
