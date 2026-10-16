use eframe::egui;
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
    let mut event = None;
    ui.separator();
    ui.label("Registered agents");
    for profile in profiles.iter_mut() {
        let id = profile["id"].as_str().unwrap_or_default();
        let parser = profile["parser"].as_str().unwrap_or("custom");
        if id.is_empty() {
            continue;
        }
        let saved = snapshot["config"]["profiles"]
            .as_array()
            .is_some_and(|saved| saved.iter().any(|item| item["id"] == id));
        let models = profile["catalog"]["models"].as_array().map_or(0, Vec::len);
        let mut open = false;
        let removal_busy =
            metadata_active(snapshot, "quota", id) || metadata_active(snapshot, "catalog", id);
        ui.horizontal(|ui| {
            ui.strong(id);
            if ui
                .add_enabled(editable, egui::Button::new("Edit"))
                .accessible_name(format!("Edit agent {id}"))
                .clicked()
            {
                open = true;
            }
            if ui
                .add_enabled(
                    editable && editor.id.is_none() && !removal_busy,
                    egui::Button::new("🗑"),
                )
                .accessible_name(format!("Remove agent {id}"))
                .clicked()
            {
                editor.request_remove(id);
            }
            ui.label(format!("{models} models"));
            if let Some(badge) = snapshot["quota_badges"][id]["text"].as_str() {
                ui.label(badge).on_hover_text(
                    snapshot["quota_badges"][id]["title"]
                        .as_str()
                        .unwrap_or_default(),
                );
            }
            let catalog_busy = metadata_active(snapshot, "catalog", id);
            if parser != "custom"
                && ui
                    .add_enabled(
                        editable && saved && !catalog_busy,
                        egui::Button::new("Models ↻"),
                    )
                    .accessible_name(format!("Refresh models for {id}"))
                    .clicked()
            {
                event = Some(Event::Discover(id.into()));
            }
            if matches!(parser, "claude" | "codex") {
                let busy = metadata_active(snapshot, "quota", id);
                if ui
                    .add_enabled(editable && saved && !busy, egui::Button::new("↻"))
                    .accessible_name(format!("Refresh quota for {id}"))
                    .on_hover_text(format!("Refresh quota for {id}"))
                    .clicked()
                {
                    event = Some(Event::Quota(id.into()));
                }
                if busy {
                    ui.spinner();
                }
            }
            if catalog_busy {
                ui.spinner();
            }
        });
        if open {
            editor.open(profile);
        }
    }
    if let Some(id) = editor.remove_confirmation.clone() {
        ui.separator();
        ui.label(format!(
            "Remove agent {id}? Its coordinator, role, and fallback assignments will be cleared."
        ));
        ui.horizontal(|ui| {
            if ui
                .add_enabled(editable, egui::Button::new("Confirm removal"))
                .accessible_name(format!("Confirm remove agent {id}"))
                .clicked()
                && remove_profile(profiles, coordinator, roles, editor, &id)
            {
                event = Some(Event::SaveProfiles);
            }
            if ui
                .button("Cancel removal")
                .accessible_name(format!("Cancel remove agent {id}"))
                .clicked()
            {
                editor.remove_confirmation = None;
            }
        });
    }
    if let Some(id) = editor.id.clone() {
        ui.separator();
        ui.strong(format!("Edit agent {id}"));
        let mut selected = None;
        ui.add_enabled_ui(editable, |ui| {
            egui::ComboBox::from_label("Provider")
                .selected_text(editor.base_profile["parser"].as_str().unwrap_or("custom"))
                .show_ui(ui, |ui| {
                    if let Some(templates) = snapshot["templates"].as_array() {
                        for template in templates {
                            let parser = template["parser"].as_str().unwrap_or("custom");
                            if ui
                                .selectable_label(editor.base_profile["parser"] == parser, parser)
                                .clicked()
                            {
                                selected = Some(template.clone());
                            }
                        }
                    }
                    if ui
                        .selectable_label(editor.base_profile["parser"] == "custom", "custom")
                        .clicked()
                    {
                        selected = Some(serde_json::json!({
                            "id":"local-agent", "parser":"custom", "executable":"aipro",
                            "args":[], "input_mode":"stdin", "permission_mode":"unverified",
                            "max_sessions":3, "quota_refresh_minutes":5, "catalog":null
                        }));
                    }
                });
        });
        if let Some(template) = selected {
            let old_executable = editor.executable().to_owned();
            editor.switch_provider(&template, profiles);
            if old_executable != editor.executable() {
                event = Some(Event::ResolveExecutable(editor.executable().to_owned()));
            }
        }
        ui.horizontal(|ui| {
            ui.label("Executable");
            ui.text_edit_singleline(&mut editor.executable);
            if ui
                .button("Browse…")
                .accessible_name(format!("Browse executable for {id}"))
                .clicked()
            {
                event = Some(Event::BrowseExecutable);
            }
        });
        ui.label("Arguments (JSON array)");
        ui.text_edit_multiline(&mut editor.args);
        ui.horizontal(|ui| {
            egui::ComboBox::from_label("Input mode")
                .selected_text(&editor.input_mode)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut editor.input_mode, "stdin".into(), "stdin");
                    ui.selectable_value(&mut editor.input_mode, "file".into(), "file");
                });
            ui.add(egui::Slider::new(&mut editor.max_sessions, 1..=3).text("Sessions"));
            ui.add(
                egui::Slider::new(&mut editor.quota_refresh_minutes, 1..=60)
                    .text("Quota refresh (min)"),
            );
        });
        let parsed = serde_json::from_str::<Vec<String>>(&editor.args);
        editor.error = parsed
            .as_ref()
            .err()
            .map(|error| format!("Invalid arguments: {error}"));
        if let Some(error) = &editor.error {
            ui.colored_label(egui::Color32::DARK_RED, error);
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    editable && parsed.is_ok(),
                    egui::Button::new("Save profile"),
                )
                .clicked()
                && let Some(profile) = profiles
                    .iter_mut()
                    .find(|profile| profile["id"] == id || profile["id"] == editor.origin["id"])
            {
                editor
                    .apply(profile)
                    .expect("validated arguments remain valid");
                event = Some(Event::SaveProfiles);
            }
            if ui.button("Cancel").clicked() {
                editor.cancel();
            }
        });
    }
    event
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

pub fn metadata_running(snapshot: &Value) -> bool {
    snapshot["metadata_job"]["state"] == "running"
        || snapshot["metadata_queue"]
            .as_array()
            .is_some_and(|queue| !queue.is_empty())
}

pub(super) fn metadata_active(snapshot: &Value, kind: &str, agent: &str) -> bool {
    let operation = format!("{kind}:{agent}");
    (snapshot["metadata_job"]["kind"] == operation
        && snapshot["metadata_job"]["state"] == "running")
        || snapshot["metadata_queue"].as_array().is_some_and(|queue| {
            queue
                .iter()
                .any(|item| item["kind"] == kind && item["agent"] == agent)
        })
}
