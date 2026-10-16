use eframe::egui;
use serde_json::Value;

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Discover(String),
    Quota(String),
    SaveProfiles,
    BrowseExecutable,
}

#[derive(Default)]
pub struct Editor {
    id: Option<String>,
    pub(super) executable: String,
    args: String,
    input_mode: String,
    max_sessions: u64,
    quota_refresh_minutes: u64,
    error: Option<String>,
}

impl Editor {
    pub fn set_executable(&mut self, path: &str) {
        self.executable = path.to_owned();
    }

    pub fn apply(&self, profile: &mut Value) -> Result<(), String> {
        let args =
            serde_json::from_str::<Vec<String>>(&self.args).map_err(|error| error.to_string())?;
        if profile["executable"].as_str().unwrap_or_default() != self.executable {
            profile["catalog"] = Value::Null;
        }
        profile["executable"] = self.executable.clone().into();
        profile["args"] = serde_json::to_value(args).unwrap();
        profile["input_mode"] = self.input_mode.clone().into();
        profile["max_sessions"] = self.max_sessions.into();
        profile["quota_refresh_minutes"] = self.quota_refresh_minutes.into();
        Ok(())
    }
    pub fn open(&mut self, profile: &Value) {
        self.id = profile["id"].as_str().map(str::to_owned);
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

pub fn metadata_request(event: &Event, revision: &str) -> Vec<u8> {
    let (kind, agent) = match event {
        Event::Discover(agent) => ("discover", agent),
        Event::Quota(agent) => ("quota", agent),
        Event::SaveProfiles | Event::BrowseExecutable => unreachable!(),
    };
    serde_json::to_vec(&serde_json::json!({"kind":kind,"agent":agent,"revision":revision}))
        .expect("metadata request is serializable")
}

pub fn profile_save_request(snapshot: &Value, profiles: &[Value]) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "kind":"save_profiles",
        "revision":snapshot["revision"],
        "profiles":profiles,
        "renames":{}
    }))
    .expect("profile save request is serializable")
}

pub fn show(
    profiles: &mut [Value],
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
        ui.horizontal(|ui| {
            ui.strong(id);
            if ui
                .add_enabled(editable, egui::Button::new("Edit"))
                .accessible_name(format!("Edit agent {id}"))
                .clicked()
            {
                open = true;
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
    if let Some(id) = editor.id.clone() {
        ui.separator();
        ui.strong(format!("Edit agent {id}"));
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
            {
                if let Some(profile) = profiles.iter_mut().find(|profile| profile["id"] == id) {
                    editor
                        .apply(profile)
                        .expect("validated arguments remain valid");
                    event = Some(Event::SaveProfiles);
                }
                editor.id = None;
            }
            if ui.button("Cancel").clicked() {
                editor.id = None;
            }
        });
    }
    event
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

fn metadata_active(snapshot: &Value, kind: &str, agent: &str) -> bool {
    let operation = format!("{kind}:{agent}");
    (snapshot["metadata_job"]["kind"] == operation
        && snapshot["metadata_job"]["state"] == "running")
        || snapshot["metadata_queue"].as_array().is_some_and(|queue| {
            queue
                .iter()
                .any(|item| item["kind"] == kind && item["agent"] == agent)
        })
}
