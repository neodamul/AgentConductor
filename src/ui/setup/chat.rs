//! Coordinator chat from the workspace screen (GitHub #115): a worker job like MCP preview,
//! with the answer stored as an artifact and returned in the next snapshots.
use super::*;

impl Startup {
    pub(super) fn launch_chat(
        &self,
        s: &mut Saved,
        turns: Vec<crate::chat::Turn>,
    ) -> Result<serde_json::Value, String> {
        let language = if crate::ui::language::load() == "ko" {
            "Korean"
        } else {
            "English"
        };
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
        self.launch(
            s,
            "chat",
            vec!["chat".into(), self.broker.dir.to_string_lossy().into(), id],
        )
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
