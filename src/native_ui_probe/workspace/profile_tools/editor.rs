use super::helpers::switch_profile;
use serde_json::Value;

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Discover(String),
    Quota(String),
    SaveProfiles,
    BrowseExecutable,
    ResolveExecutable(String),
}

#[derive(Default)]
pub struct Editor {
    pub(super) id: Option<String>,
    pub(super) remove_confirmation: Option<String>,
    pub(super) origin: Value,
    pub(super) base_profile: Value,
    pub(super) renames: std::collections::BTreeMap<String, String>,
    pub(super) executable: String,
    pub(super) args: String,
    pub(super) extensions: String,
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

    pub fn set_executable(&mut self, path: &str) {
        self.executable = path.to_owned();
    }

    pub fn executable(&self) -> &str {
        &self.executable
    }

    pub fn apply_resolution(&mut self, requested: &str, resolved: Option<&str>) {
        if self.executable == requested
            && let Some(path) = resolved
        {
            self.executable = path.to_owned();
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
    }

    pub fn cancel(&mut self) {
        self.renames.clear();
        self.id = None;
        self.remove_confirmation = None;
    }

    pub fn switch_provider(&mut self, template: &Value, profiles: &[Value]) {
        if self.base_profile["parser"] == template["parser"] {
            return;
        }
        if let Some((profile, renames)) = switch_profile(&self.origin, template, profiles) {
            let origin = self.origin.clone();
            self.open(&profile);
            self.origin = origin;
            self.renames = renames;
        }
    }

    pub fn apply(&self, profile: &mut Value) -> Result<(), String> {
        let args =
            serde_json::from_str::<Vec<String>>(&self.args).map_err(|error| error.to_string())?;
        let changed = profile["executable"] != self.executable
            || profile["parser"] != self.base_profile["parser"];
        let mut candidate = self.base_profile.clone();
        candidate["executable"] = self.executable.clone().into();
        candidate["args"] = serde_json::to_value(args).unwrap();
        candidate["input_mode"] = self.input_mode.clone().into();
        candidate["max_sessions"] = self.max_sessions.into();
        candidate["quota_refresh_minutes"] = self.quota_refresh_minutes.into();
        let extensions: Value = serde_json::from_str(&self.extensions)
            .map_err(|error| format!("invalid profile extensions: {error}"))?;
        if !extensions.is_object() {
            return Err("profile extensions must be a JSON object".into());
        }
        for key in [
            "catalog",
            "model_discovery",
            "execution_options",
            "result_pointer",
            "error_condition",
        ] {
            if let Some(value) = extensions.get(key) {
                candidate[key] = if key == "execution_options" && value.is_null() {
                    serde_json::json!([])
                } else {
                    value.clone()
                };
            }
        }
        if changed {
            candidate["catalog"] = Value::Null;
        } else if candidate["catalog"]["models"] != self.base_profile["catalog"]["models"]
            && let Some(catalog) = candidate["catalog"].as_object_mut()
        {
            catalog.insert("origin".into(), "manual".into());
            catalog.insert("source".into(), "operator-configured catalog".into());
            catalog.insert("queried_at".into(), 0.into());
            catalog.insert("cli_version".into(), "manual".into());
        }
        let parsed: crate::plan::AgentProfile = serde_json::from_value(candidate.clone())
            .map_err(|error| format!("invalid profile configuration: {error}"))?;
        parsed.validate_custom_configuration()?;
        if let Some(model) = parsed
            .execution_options
            .iter()
            .find(|option| option.name == "model")
            .and_then(|option| option.default.as_deref())
        {
            parsed.validate_model(model)?;
        }
        *profile = candidate;
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
        self.base_profile = profile.clone();
        self.executable = profile["executable"].as_str().unwrap_or_default().into();
        self.args = serde_json::to_string_pretty(&profile["args"]).unwrap_or_else(|_| "[]".into());
        self.extensions = serde_json::to_string_pretty(&serde_json::json!({
            "catalog":profile["catalog"], "model_discovery":profile["model_discovery"],
            "execution_options":profile["execution_options"], "result_pointer":profile["result_pointer"],
            "error_condition":profile["error_condition"]
        })).unwrap_or_else(|_| "{}".into());
        self.input_mode = profile["input_mode"].as_str().unwrap_or("stdin").into();
        self.max_sessions = profile["max_sessions"].as_u64().unwrap_or(1).clamp(1, 3);
        self.quota_refresh_minutes = profile["quota_refresh_minutes"]
            .as_u64()
            .unwrap_or(5)
            .clamp(1, 60);
        self.error = None;
    }
}
