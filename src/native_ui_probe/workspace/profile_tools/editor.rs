use super::{Event, switch_profile};
use serde_json::Value;

#[derive(Default)]
pub struct Editor {
    pub(super) id: Option<String>,
    pub(super) remove_confirmation: Option<String>,
    pub(super) origin: Value,
    pub(super) base_profile: Value,
    renames: std::collections::BTreeMap<String, String>,
    pub(super) agent_id: String,
    pub(super) fallback_agent: Option<String>,
    pub(super) fallback_model: Option<String>,
    fallback_catalog_requested: Option<String>,
    pub(super) applied: bool,
    pub(super) executable: String,
    pub(super) args: String,
    pub(super) input_mode: String,
    pub(super) max_sessions: u64,
    pub(super) quota_refresh_minutes: u64,
    pub(super) error: Option<String>,
}

impl Editor {
    pub fn request_remove(&mut self, id: &str) {
        self.remove_confirmation = Some(id.to_owned());
    }

    pub fn is_editing(&self) -> bool {
        self.id.is_some()
    }

    pub fn set_agent_id(&mut self, id: &str) {
        self.agent_id = id.to_owned();
        self.error = None;
    }

    pub fn agent_id_is_unique(&self, profiles: &[Value]) -> bool {
        let matching_ids = profiles
            .iter()
            .filter(|profile| profile["id"] == self.agent_id)
            .count();
        let current_matches = if self.applied {
            profiles.iter().any(|profile| {
                profile["id"] == self.agent_id
                    && profile["parser"] == self.base_profile["parser"]
                    && profile["executable"] == self.executable
            })
        } else {
            self.agent_id == self.origin["id"]
        };
        matching_ids <= usize::from(current_matches)
    }

    pub fn set_fallback(&mut self, agent: Option<&str>, model: Option<&str>) {
        let agent = agent.map(str::to_owned);
        if self.fallback_agent != agent {
            self.fallback_model = None;
            self.fallback_catalog_requested = None;
        }
        self.fallback_agent = agent;
        self.fallback_model = model.map(str::to_owned);
        self.error = None;
    }

    pub fn fallback_catalog_event(
        &mut self,
        profiles: &[Value],
        saved_profiles: &[Value],
    ) -> Option<Event> {
        let agent = self.fallback_agent.as_ref()?;
        if self.fallback_catalog_requested.as_ref() == Some(agent)
            || !saved_profiles
                .iter()
                .any(|profile| profile["id"].as_str() == Some(agent.as_str()))
        {
            return None;
        }
        let profile = profiles
            .iter()
            .find(|profile| profile["id"].as_str() == Some(agent.as_str()))?;
        if profile["parser"] == "custom"
            || profile["catalog"]["models"]
                .as_array()
                .is_some_and(|models| !models.is_empty())
        {
            return None;
        }
        self.fallback_catalog_requested = Some(agent.clone());
        Some(Event::Discover(agent.clone()))
    }

    pub fn set_executable(&mut self, path: &str) {
        self.executable = path.to_owned();
        self.error = None;
    }

    pub fn executable(&self) -> &str {
        &self.executable
    }

    pub fn apply_resolution(&mut self, requested: &str, resolved: Option<&str>) {
        if self.executable == requested
            && let Some(path) = resolved
        {
            self.set_executable(path);
        }
    }

    pub fn renames(&self) -> &std::collections::BTreeMap<String, String> {
        &self.renames
    }

    pub fn clear_renames(&mut self) {
        self.renames.clear();
    }

    pub fn finish_save(&mut self) {
        self.renames.clear();
        self.id = None;
        self.error = None;
    }

    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
    }

    pub fn set_save_error(&mut self, language: &str, detail: &str) {
        let message = crate::ui::messages::in_language(language, "startup.profile_save_failed");
        self.set_error(crate::ui::messages::interpolate(
            message,
            &[("value1", detail)],
        ));
    }

    pub fn cancel(&mut self) {
        self.renames.clear();
        self.id = None;
        self.remove_confirmation = None;
        self.error = None;
    }

    pub fn switch_provider(&mut self, template: &Value, profiles: &[Value]) {
        if self.base_profile["parser"] == template["parser"] {
            return;
        }
        if let Some((mut profile, renames)) = switch_profile(&self.origin, template, profiles) {
            let origin = self.origin.clone();
            for key in ["environment", "fallback"] {
                if !origin[key].is_null() {
                    profile[key] = origin[key].clone();
                }
            }
            self.open(&profile);
            self.origin = origin;
            self.renames = renames;
        }
    }

    pub fn apply(&mut self, profile: &mut Value) -> Result<(), String> {
        let args =
            serde_json::from_str::<Vec<String>>(&self.args).map_err(|error| error.to_string())?;
        if self.agent_id.trim().is_empty() {
            return Err("agent ID cannot be empty".into());
        }
        let changed = profile["executable"] != self.executable
            || profile["parser"] != self.base_profile["parser"];
        *profile = self.base_profile.clone();
        profile["id"] = self.agent_id.clone().into();
        profile["executable"] = self.executable.clone().into();
        profile["args"] = serde_json::to_value(args).unwrap();
        profile["input_mode"] = self.input_mode.clone().into();
        profile["max_sessions"] = self.max_sessions.into();
        profile["quota_refresh_minutes"] = self.quota_refresh_minutes.into();
        profile["fallback"] = self.fallback_agent.as_ref().map_or(
            Value::Null,
            |agent| serde_json::json!({"agent":agent,"model":self.fallback_model}),
        );
        self.renames.clear();
        if let Some(previous) = self.origin["id"].as_str()
            && previous != self.agent_id
        {
            self.renames
                .insert(previous.to_owned(), self.agent_id.clone());
        }
        if changed {
            profile["catalog"] = Value::Null;
        }
        self.id = Some(self.agent_id.clone());
        self.applied = true;
        Ok(())
    }

    pub fn open(&mut self, profile: &Value) {
        self.origin = profile.clone();
        self.renames.clear();
        self.remove_confirmation = None;
        self.load(profile);
    }

    fn load(&mut self, profile: &Value) {
        self.id = profile["id"].as_str().map(str::to_owned);
        self.applied = false;
        self.agent_id = profile["id"].as_str().unwrap_or_default().into();
        self.fallback_agent = profile["fallback"]["agent"].as_str().map(str::to_owned);
        self.fallback_model = profile["fallback"]["model"].as_str().map(str::to_owned);
        self.fallback_catalog_requested = None;
        self.base_profile = profile.clone();
        self.executable = profile["executable"].as_str().unwrap_or_default().into();
        self.args = serde_json::to_string_pretty(&profile["args"]).unwrap_or_else(|_| "[]".into());
        self.input_mode = profile["input_mode"].as_str().unwrap_or("stdin").into();
        self.max_sessions = profile["max_sessions"].as_u64().unwrap_or(1).clamp(1, 3);
        self.quota_refresh_minutes = profile["quota_refresh_minutes"]
            .as_u64()
            .unwrap_or(5)
            .clamp(1, 60);
        self.error = None;
    }
}
