//! Coordinator chat from the workspace screen (GitHub #115): a worker job like MCP preview,
//! with the answer stored as an artifact and returned in the next snapshots.
use super::*;

impl Startup {
    pub(super) fn dispatch_chat(
        &self,
        saved: &mut Saved,
        revision: &str,
        text: &str,
    ) -> Result<serde_json::Value, String> {
        // Chat cannot change repository state, so bypass the Git HEAD revision guard.
        let job = self.job(saved)?;
        self.settle(saved, &job)?;
        if job["state"] == "running" {
            return Err(crate::ui::messages::text("native.startup_operation_still_running").into());
        }
        self.ensure_no_live_jobs()?;
        self.launch_chat(saved, text.to_owned(), revision)
    }

    pub(super) fn launch_chat(
        &self,
        s: &mut Saved,
        text: String,
        revision: &str,
    ) -> Result<serde_json::Value, String> {
        let language = if crate::ui::language::load() == "ko" {
            "Korean"
        } else {
            "English"
        };
        let history = s
            .chat_result
            .as_deref()
            .map(|name| crate::chat::history(&self.broker.dir, name))
            .unwrap_or_default();
        let turns = crate::chat::next_turns(history, &text)?;
        crate::chat::build_prompt(&turns, language)?;
        crate::mcp::coordinator_profile(&s.config.profiles, &s.config.coordinator)?;
        let id = crate::native_guard::session_id()?;
        crate::mcp::store(
            &self.broker.dir,
            &format!("chat-input-{id}.json"),
            &serde_json::to_vec(&serde_json::json!({"turns": turns, "language": language}))
                .map_err(|e| e.to_string())?,
        )?;
        s.chat_result = Some(format!("chat-{id}.json"));
        self.launch_with_revision(
            s,
            "chat",
            vec!["chat".into(), self.broker.dir.to_string_lossy().into(), id],
            revision,
        )
    }

    /// The stored conversation of this workspace, so a reopened screen shows it again.
    pub(super) fn chat_history(&self, s: &Saved) -> serde_json::Value {
        let turns = s
            .chat_result
            .as_deref()
            .map(|name| crate::chat::history(&self.broker.dir, name))
            .unwrap_or_default();
        serde_json::json!({"turns": turns})
    }

    /// Starts a new conversation; the old artifacts stay in the private folder.
    pub(super) fn chat_reset(
        &self,
        s: &mut Saved,
        job: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        if job["kind"] == "chat" && job["state"] == "running" {
            return Err(crate::ui::messages::text("native.startup_operation_still_running").into());
        }
        s.chat_result = None;
        self.store(s)?;
        Ok(serde_json::json!({"turns": []}))
    }

    /// The finished answer, tagged with its job so the page shows each answer once.
    pub(super) fn chat_snapshot(&self, s: &Saved, job: &serde_json::Value) -> serde_json::Value {
        if job["kind"] != "chat" || job["state"] != "completed" {
            return serde_json::Value::Null;
        }
        let Some(name) = &s.chat_result else {
            return serde_json::Value::Null;
        };
        crate::mcp::load(&self.broker.dir.join("mcp").join(name))
            .ok()
            .and_then(|data| serde_json::from_slice::<serde_json::Value>(&data).ok())
            .map_or(
                serde_json::Value::Null,
                |v| serde_json::json!({"job": job["id"], "answer": v["answer"]}),
            )
    }
}
