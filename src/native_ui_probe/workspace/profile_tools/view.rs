use super::{
    Editor, Event,
    helpers::{metadata_active, remove_profile},
};
use eframe::egui;
use serde_json::Value;

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
            if (parser != "custom" || !profile["model_discovery"].is_null())
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
        ui.label("Catalog and execution rules (JSON)");
        ui.label(r#"Manual catalog: {"models":["model-id"],"source":"operator","queried_at":0,"cli_version":"manual","origin":"manual"}"#);
        ui.text_edit_multiline(&mut editor.extensions);
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
        let extensions = serde_json::from_str::<Value>(&editor.extensions);
        editor.error = parsed
            .as_ref()
            .err()
            .map(|error| format!("Invalid arguments: {error}"))
            .or_else(|| {
                extensions
                    .as_ref()
                    .err()
                    .map(|error| format!("Invalid profile rules: {error}"))
            })
            .or_else(|| {
                extensions.as_ref().ok().and_then(|value| {
                    (!value.is_object()).then_some("Profile rules must be a JSON object".into())
                })
            });
        if let Some(error) = &editor.error {
            ui.colored_label(egui::Color32::DARK_RED, error);
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    editable
                        && parsed.is_ok()
                        && extensions.is_ok()
                        && extensions.as_ref().is_ok_and(Value::is_object),
                    egui::Button::new("Save profile"),
                )
                .clicked()
                && let Some(profile) = profiles
                    .iter_mut()
                    .find(|profile| profile["id"] == id || profile["id"] == editor.origin["id"])
            {
                match editor.apply(profile) {
                    Ok(()) => event = Some(Event::SaveProfiles),
                    Err(error) => editor.error = Some(error),
                }
            }
            if ui.button("Cancel").clicked() {
                editor.cancel();
            }
        });
    }
    event
}
