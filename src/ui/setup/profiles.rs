//! DD-56: store only the agent list, keeping coordinator and role rules consistent.
use super::*;

impl Startup {
    pub(super) fn save_profiles(
        &self,
        saved: &mut Saved,
        mut profiles: Vec<AgentProfile>,
    ) -> Result<serde_json::Value, String> {
        validate_profiles(&profiles)?;
        for profile in &mut profiles {
            if profile.environment.is_empty()
                && let Some(old) = saved.config.profiles.iter().find(|p| p.id == profile.id)
            {
                profile.environment = old.environment.clone();
            }
        }
        let mut next = saved.clone();
        if !profiles.iter().any(|p| p.id == next.config.coordinator) {
            next.config.coordinator = profiles.first().map(|p| p.id.clone()).unwrap_or_default();
        }
        for entries in next.config.roles.values_mut() {
            entries.retain(|e| profiles.iter().any(|p| p.id == e.agent));
        }
        next.config.roles.retain(|_, entries| !entries.is_empty());
        next.config.profiles = profiles;
        next.proposal = None;
        next.job = None;
        self.store(&next)?;
        *saved = next;
        Ok(serde_json::json!({"saved":true,"config":saved.config}))
    }
}

pub(crate) fn validate_name(name: &str) -> Result<(), String> {
    if name.chars().count() > 80 || name.chars().any(char::is_control) {
        return Err(
            "workspace name must be at most 80 characters without control characters".into(),
        );
    }
    Ok(())
}
