use super::super::*;
use super::{LOCAL_CLAIMS, read_shared};

impl Startup {
    pub(in crate::ui::setup) fn preview_settings(
        &self,
        saved: &Saved,
        dir: &Path,
    ) -> Result<serde_json::Value, String> {
        let value = read_shared(dir)?;
        let agents: Vec<AgentProfile> =
            serde_json::from_value(value["agents"].clone()).map_err(|e| {
                crate::ui::messages::render("native.invalid_agents", &[("e", &e.to_string())])
            })?;
        let mut warnings = vec![];
        for agent in &agents {
            if LOCAL_CLAIMS.contains(&agent.permission_mode.as_str()) {
                warnings.push(format!("{}: {} must be verified on this machine; it is imported as Not verified unless you confirm", agent.id, agent.permission_mode));
            }
            if Path::new(&agent.executable).is_absolute() {
                warnings.push(format!(
                    "{}: absolute executable {} may not exist on this machine",
                    agent.id, agent.executable
                ));
            }
        }
        for local in &saved.config.profiles {
            if !agents.iter().any(|a| a.id == local.id) {
                warnings.push(format!(
                    "{}: local agent is not in the shared settings and will be removed",
                    local.id
                ));
            }
        }
        Ok(serde_json::json!({"settings":value,"warnings":warnings}))
    }
}
