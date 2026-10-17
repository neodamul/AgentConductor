use super::{Editor, Event, metadata_active, remove_profile};
use eframe::egui;
use serde_json::Value;

pub struct RenderContext<'a> {
    pub snapshot: &'a Value,
    pub editable: bool,
    pub language: &'a str,
}

pub fn show_in_language(
    profiles: &mut Vec<Value>,
    coordinator: &mut String,
    roles: &mut crate::roles::RoleRules,
    editor: &mut Editor,
    context: RenderContext<'_>,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let RenderContext {
        snapshot,
        editable,
        language,
    } = context;
    let mut event = None;
    ui.separator();
    ui.label(text(language, "startup.registered_agents"))
        .accessible_name(text(language, "startup.registered_agents"));
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
                .add_enabled(editable, egui::Button::new(text(language, "startup.edit")))
                .accessible_name(message(
                    language,
                    "startup.edit_agent_named",
                    &[("value1", id)],
                ))
                .clicked()
            {
                open = true;
            }
            if ui
                .add_enabled(
                    editable && editor.id.is_none() && !removal_busy,
                    egui::Button::new("🗑"),
                )
                .accessible_name(message(
                    language,
                    "startup.remove_agent_named",
                    &[("value1", id)],
                ))
                .clicked()
            {
                editor.request_remove(id);
            }
            let model_count = message(
                language,
                "startup.models_dee84b",
                &[("value1", &models.to_string())],
            );
            ui.label(&model_count).accessible_name(model_count);
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
                        egui::Button::new(format!("{} ↻", text(language, "startup.models"))),
                    )
                    .accessible_name(message(
                        language,
                        "startup.refresh_models_for_agent",
                        &[("value1", id)],
                    ))
                    .clicked()
            {
                event = Some(Event::Discover(id.into()));
            }
            if matches!(parser, "claude" | "codex") {
                let busy = metadata_active(snapshot, "quota", id);
                if ui
                    .add_enabled(editable && saved && !busy, egui::Button::new("↻"))
                    .accessible_name(message(
                        language,
                        "startup.refresh_quota_for_agent",
                        &[("value1", id)],
                    ))
                    .on_hover_text(message(
                        language,
                        "startup.refresh_quota_for_agent",
                        &[("value1", id)],
                    ))
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
        ui.label(message(
            language,
            "startup.confirm_remove_agent_message",
            &[("value1", &id)],
        ));
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    editable,
                    egui::Button::new(text(language, "startup.confirm_remove_agent")),
                )
                .accessible_name(message(
                    language,
                    "startup.confirm_remove_agent_named",
                    &[("value1", &id)],
                ))
                .clicked()
                && remove_profile(profiles, coordinator, roles, editor, &id)
            {
                event = Some(Event::SaveProfiles);
            }
            if ui
                .button(text(language, "startup.cancel_remove_agent"))
                .accessible_name(message(
                    language,
                    "startup.cancel_remove_agent_named",
                    &[("value1", &id)],
                ))
                .clicked()
            {
                editor.remove_confirmation = None;
            }
        });
    }
    if let Some(id) = editor.id.clone() {
        ui.separator();
        ui.strong(message(
            language,
            "startup.edit_agent_named",
            &[("value1", &id)],
        ))
        .accessible_name(message(
            language,
            "startup.edit_agent_named",
            &[("value1", &id)],
        ));
        let mut selected = None;
        ui.add_enabled_ui(editable, |ui| {
            egui::ComboBox::from_label(text(language, "startup.provider"))
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
            ui.add(egui::TextEdit::singleline(&mut editor.executable))
                .accessible_name(text(language, "startup.executable"));
            if ui
                .button(text(language, "startup.browse"))
                .accessible_name(message(
                    language,
                    "startup.browse_agent_executable_named",
                    &[("value1", &id)],
                ))
                .clicked()
            {
                event = Some(Event::BrowseExecutable);
            }
        });
        ui.add(egui::TextEdit::multiline(&mut editor.args))
            .accessible_name(text(language, "startup.arguments_json_array"));
        ui.horizontal(|ui| {
            egui::ComboBox::from_label(text(language, "startup.input_mode"))
                .selected_text(&editor.input_mode)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut editor.input_mode, "stdin".into(), "stdin");
                    ui.selectable_value(&mut editor.input_mode, "file".into(), "file");
                });
            let sessions_label = text(language, "startup.profile_sessions");
            ui.add(egui::Slider::new(&mut editor.max_sessions, 1..=3).text(sessions_label))
                .accessible_name(sessions_label);
            let refresh_label = text(language, "startup.quota_refresh_minutes");
            ui.add(
                egui::Slider::new(&mut editor.quota_refresh_minutes, 1..=60).text(refresh_label),
            )
            .accessible_name(refresh_label);
        });
        let parsed = serde_json::from_str::<Vec<String>>(&editor.args);
        editor.error = parsed.as_ref().err().map(|error| {
            let error = error.to_string();
            message(
                language,
                "startup.invalid_profile_arguments",
                &[("value1", &error)],
            )
        });
        if let Some(error) = &editor.error {
            ui.colored_label(egui::Color32::DARK_RED, error);
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    editable && parsed.is_ok(),
                    egui::Button::new(text(language, "startup.save_profile")),
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
            if ui.button(text(language, "startup.cancel")).clicked() {
                editor.cancel();
            }
        });
    }
    event
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

fn message(language: &str, id: &'static str, params: &[(&str, &str)]) -> String {
    crate::ui::messages::interpolate(text(language, id), params)
}
