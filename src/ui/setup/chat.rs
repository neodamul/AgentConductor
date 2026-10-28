//! Bounded coordinator chat jobs, using the configured CLI and its normal login.
use super::*;

impl Startup {
    pub(super) fn launch_chat(
        &self,
        saved: &mut Saved,
        turns: Vec<crate::chat::Turn>,
        revision: String,
    ) -> Result<serde_json::Value, String> {
        clear_previous_chat_artifacts(&self.broker.dir)?;
        let language = if crate::ui::language::load() == "ko" {
            "Korean"
        } else {
            "English"
        };
        crate::chat::build_prompt(&turns, language)?;
        crate::mcp::coordinator_profile(&saved.config.profiles, &saved.config.coordinator)?;
        let id = crate::native_guard::session_id()?;
        crate::mcp::store(
            &self.broker.dir,
            &format!("chat-input-{id}.json"),
            &serde_json::to_vec(&serde_json::json!({"turns": turns, "language": language}))
                .map_err(|e| e.to_string())?,
        )?;
        saved.chat_result = Some(format!("chat-{id}.json"));
        self.launch_with_revision(
            saved,
            "chat",
            vec!["chat".into(), self.broker.dir.to_string_lossy().into(), id],
            revision,
        )
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
        let path = self.broker.dir.join("mcp").join(name);
        let answer = crate::mcp::load(&path)
            .ok()
            .and_then(|data| serde_json::from_slice::<serde_json::Value>(&data).ok())
            .map(|value| serde_json::json!({"job": job["id"], "answer": value["answer"]}));
        let _ = std::fs::remove_file(path);
        answer.unwrap_or(serde_json::Value::Null)
    }
}

fn clear_previous_chat_artifacts(root: &Path) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry.file_name().to_string_lossy().starts_with("chat-run-")
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
