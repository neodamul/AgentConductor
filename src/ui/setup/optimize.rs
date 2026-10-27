//! Goal rewriting from the workspace screen (GitHub #180): a worker job like the coordinator
//! chat, with the proposal stored as an artifact and returned once in the next snapshots.
use super::*;

impl Startup {
    pub(super) fn launch_optimize(
        &self,
        s: &mut Saved,
        goal: String,
    ) -> Result<serde_json::Value, String> {
        let language = if crate::ui::language::load() == "ko" {
            "Korean"
        } else {
            "English"
        };
        crate::prompt_optimizer::build_prompt(&goal, language)?;
        crate::mcp::coordinator_profile(&s.config.profiles, &s.config.coordinator)?;
        let id = crate::native_guard::session_id()?;
        crate::mcp::store(
            &self.broker.dir,
            &format!("optimize-input-{id}.json"),
            &serde_json::to_vec(&serde_json::json!({"goal": goal, "language": language}))
                .map_err(|e| e.to_string())?,
        )?;
        s.optimize_result = Some(format!("optimize-{id}.json"));
        self.launch(
            s,
            "optimize",
            vec![
                "optimize".into(),
                self.broker.dir.to_string_lossy().into(),
                id,
            ],
        )
    }

    /// The finished proposal, tagged with its job so the page shows each one once.
    pub(super) fn optimize_snapshot(
        &self,
        s: &Saved,
        job: &serde_json::Value,
    ) -> serde_json::Value {
        if job["kind"] != "optimize" || job["state"] != "completed" {
            return serde_json::Value::Null;
        }
        let Some(name) = &s.optimize_result else {
            return serde_json::Value::Null;
        };
        crate::mcp::load(&self.broker.dir.join("mcp").join(name))
            .ok()
            .and_then(|data| serde_json::from_slice::<serde_json::Value>(&data).ok())
            .map_or(
                serde_json::Value::Null,
                |v| serde_json::json!({"job": job["id"], "proposal": v}),
            )
    }
}
