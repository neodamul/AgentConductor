//! Bounded coordinator chat jobs, using the configured CLI and its normal login.
use super::*;

impl Startup {
    pub(super) fn launch_chat(
        &self,
        saved: &mut Saved,
        question: String,
    ) -> Result<serde_json::Value, String> {
        if question.trim().is_empty() {
            return Err("the coordinator question is empty".into());
        }
        saved.chat_transcript.push(crate::chat::Turn {
            role: "operator".into(),
            text: question,
        });
        if saved.chat_transcript.len() > 80 {
            saved
                .chat_transcript
                .drain(..saved.chat_transcript.len() - 80);
        }
        let language = if crate::ui::language::load() == "ko" {
            "Korean"
        } else {
            "English"
        };
        crate::chat::build_prompt(&saved.chat_transcript, language)?;
        crate::mcp::coordinator_profile(&saved.config.profiles, &saved.config.coordinator)?;
        let id = crate::native_guard::session_id()?;
        crate::mcp::store(
            &self.broker.dir,
            &format!("chat-input-{id}.json"),
            &serde_json::to_vec(
                &serde_json::json!({"turns": &saved.chat_transcript, "language": language}),
            )
            .map_err(|e| e.to_string())?,
        )?;
        saved.chat_result = Some(format!("chat-{id}.json"));
        self.launch(
            saved,
            "chat",
            vec!["chat".into(), self.broker.dir.to_string_lossy().into(), id],
        )
    }

    pub(super) fn chat_snapshot(
        &self,
        saved: &mut Saved,
        job: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        if job["kind"] != "chat" || job["state"] != "completed" {
            return Ok(serde_json::Value::Null);
        }
        let Some(name) = &saved.chat_result else {
            return Ok(serde_json::Value::Null);
        };
        let path = self.broker.dir.join("mcp").join(name);
        let answer = crate::mcp::load(&path)
            .ok()
            .and_then(|data| serde_json::from_slice::<serde_json::Value>(&data).ok())
            .map(|value| serde_json::json!({"job": job["id"], "answer": value["answer"]}));
        if let Some(answer) = answer.and_then(|value| value["answer"].as_str().map(str::to_owned)) {
            saved.chat_transcript.push(crate::chat::Turn {
                role: "assistant".into(),
                text: answer.clone(),
            });
            saved.chat_result = None;
            self.store(saved)?;
            let _ = std::fs::remove_file(path);
            Ok(
                serde_json::json!({"job": job["id"], "answer": answer, "turns": saved.chat_transcript}),
            )
        } else {
            Ok(serde_json::Value::Null)
        }
    }
}
