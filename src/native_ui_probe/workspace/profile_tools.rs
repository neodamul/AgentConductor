use eframe::egui;
use serde_json::Value;

mod metadata;
mod view;
pub(super) use metadata::metadata_active;
pub use metadata::metadata_running;
pub use view::{RenderContext, show_in_language};

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
    id: Option<String>,
    remove_confirmation: Option<String>,
    origin: Value,
    base_profile: Value,
    renames: std::collections::BTreeMap<String, String>,
    executable: String,
    args: String,
    input_mode: String,
    max_sessions: u64,
    quota_refresh_minutes: u64,
    error: Option<String>,
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
        *profile = self.base_profile.clone();
        profile["executable"] = self.executable.clone().into();
        profile["args"] = serde_json::to_value(args).unwrap();
        profile["input_mode"] = self.input_mode.clone().into();
        profile["max_sessions"] = self.max_sessions.into();
        profile["quota_refresh_minutes"] = self.quota_refresh_minutes.into();
        if changed {
            profile["catalog"] = Value::Null;
        }
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
        self.input_mode = profile["input_mode"].as_str().unwrap_or("stdin").into();
        self.max_sessions = profile["max_sessions"].as_u64().unwrap_or(1).clamp(1, 3);
        self.quota_refresh_minutes = profile["quota_refresh_minutes"]
            .as_u64()
            .unwrap_or(5)
            .clamp(1, 60);
        self.error = None;
    }
}

pub fn switch_profile(
    current: &Value,
    template: &Value,
    profiles: &[Value],
) -> Option<(Value, std::collections::BTreeMap<String, String>)> {
    let old_id = current["id"].as_str()?;
    let base = template["id"].as_str()?;
    let mut id = base.to_owned();
    let mut suffix = 2;
    while profiles
        .iter()
        .any(|profile| profile["id"] == id && profile["id"] != old_id)
    {
        id = format!("{base}-{suffix}");
        suffix += 1;
    }
    let mut switched = template.clone();
    switched["id"] = id.clone().into();
    let renames = if old_id != id {
        [(old_id.to_owned(), id)].into_iter().collect()
    } else {
        Default::default()
    };
    Some((switched, renames))
}

pub fn rename_references(
    profiles: &mut [Value],
    coordinator: &mut String,
    roles: &mut crate::roles::RoleRules,
    renames: &std::collections::BTreeMap<String, String>,
) {
    let rename = |id: &mut String| {
        if let Some(next) = renames.get(id) {
            *id = next.clone();
        }
    };
    rename(coordinator);
    for entries in roles.values_mut() {
        for entry in entries {
            rename(&mut entry.agent);
        }
    }
    for profile in profiles {
        if let Some(id) = profile["fallback"]["agent"].as_str().map(str::to_owned) {
            let mut id = id;
            rename(&mut id);
            profile["fallback"]["agent"] = id.into();
        }
    }
}

pub fn metadata_request(event: &Event, revision: &str) -> Vec<u8> {
    let (kind, agent) = match event {
        Event::Discover(agent) => ("discover", agent),
        Event::Quota(agent) => ("quota", agent),
        Event::SaveProfiles | Event::BrowseExecutable | Event::ResolveExecutable(_) => {
            unreachable!()
        }
    };
    serde_json::to_vec(&serde_json::json!({"kind":kind,"agent":agent,"revision":revision}))
        .expect("metadata request is serializable")
}

pub fn profile_save_request(
    snapshot: &Value,
    profiles: &[Value],
    renames: &std::collections::BTreeMap<String, String>,
) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "kind":"save_profiles",
        "revision":snapshot["revision"],
        "profiles":profiles,
        "renames":renames
    }))
    .expect("profile save request is serializable")
}

pub fn show(
    profiles: &mut Vec<Value>,
    coordinator: &mut String,
    roles: &mut crate::roles::RoleRules,
    snapshot: &Value,
    editable: bool,
    editor: &mut Editor,
    ui: &mut egui::Ui,
) -> Option<Event> {
    view::show_in_language(
        profiles,
        coordinator,
        roles,
        editor,
        RenderContext {
            snapshot,
            editable,
            language: "en",
        },
        ui,
    )
}

pub fn remove_profile(
    profiles: &mut Vec<Value>,
    coordinator: &mut String,
    roles: &mut crate::roles::RoleRules,
    editor: &mut Editor,
    id: &str,
) -> bool {
    let length = profiles.len();
    profiles.retain(|profile| profile["id"] != id);
    if profiles.len() == length {
        editor.remove_confirmation = None;
        return false;
    }
    if editor.id.as_deref() == Some(id) {
        editor.cancel();
    }
    editor.remove_confirmation = None;
    remove_references(profiles, coordinator, roles, id);
    true
}

pub fn remove_references(
    profiles: &mut [Value],
    coordinator: &mut String,
    roles: &mut crate::roles::RoleRules,
    removed: &str,
) {
    if coordinator == removed {
        *coordinator = profiles
            .first()
            .and_then(|profile| profile["id"].as_str())
            .unwrap_or_default()
            .to_owned();
    }
    for profile in profiles {
        if profile["fallback"]["agent"] == removed {
            profile["fallback"] = Value::Null;
        }
    }
    roles.retain(|_, entries| {
        entries.retain(|entry| entry.agent != removed);
        !entries.is_empty()
    });
}

pub fn merge_catalogs(local: &mut [Value], saved: &[Value]) {
    for profile in local {
        let Some(observed) = saved.iter().find(|item| {
            item["id"] == profile["id"]
                && item["parser"] == profile["parser"]
                && item["executable"] == profile["executable"]
        }) else {
            continue;
        };
        if !observed["catalog"].is_null() {
            profile["catalog"] = observed["catalog"].clone();
        }
    }
}
