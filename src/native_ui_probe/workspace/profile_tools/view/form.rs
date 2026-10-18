use super::{message, text};
use crate::native_ui_probe::workspace::profile_tools::{Editor, Event};
use eframe::egui;
use serde_json::Value;

mod fallback;

pub(super) fn show(
    profiles: &mut [Value],
    snapshot: &Value,
    editable: bool,
    editor: &mut Editor,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Event> {
    editor.id.as_ref()?;

    let (mut event, selected, id_changed) = egui::ScrollArea::vertical()
        .max_height((ui.ctx().content_rect().height() - 160.0).max(100.0))
        .show(ui, |ui| {
            let mut event = None;
            let mut selected = None;
            let mut id_changed = false;
            ui.columns(2, |columns| {
                let (left, right) = columns.split_at_mut(1);
                let left = &mut left[0];
                let right = &mut right[0];
                let provider_label = text(language, "startup.provider");
                left.label(provider_label);
                left.add_enabled_ui(editable, |ui| {
                    egui::ComboBox::from_id_salt("native-agent-provider")
                        .selected_text(editor.base_profile["parser"].as_str().unwrap_or("custom"))
                        .width(ui.available_width())
                        .show_ui(ui, |ui| {
                            if let Some(templates) = snapshot["templates"].as_array() {
                                for template in templates {
                                    let parser = template["parser"].as_str().unwrap_or("custom");
                                    if ui
                                        .selectable_label(
                                            editor.base_profile["parser"] == parser,
                                            parser,
                                        )
                                        .clicked()
                                    {
                                        selected = Some(template.clone());
                                    }
                                }
                            }
                            if ui
                                .selectable_label(
                                    editor.base_profile["parser"] == "custom",
                                    "custom",
                                )
                                .clicked()
                            {
                                selected = Some(super::super::custom_template());
                            }
                        })
                        .response
                        .accessible_name(provider_label);
                });
                let id_label = text(language, "startup.agent_id");
                right.label(id_label);
                id_changed = right
                    .add_enabled_ui(editable, |ui| {
                        ui.add(egui::TextEdit::singleline(&mut editor.agent_id))
                            .accessible_name(id_label)
                            .changed()
                    })
                    .inner;
            });
            let executable_label = text(language, "startup.executable");
            ui.label(executable_label);
            ui.horizontal(|ui| {
                let width = (ui.available_width() - 88.0).max(100.0);
                let response = ui.add_enabled(
                    editable,
                    egui::TextEdit::singleline(&mut editor.executable).desired_width(width),
                );
                if response.changed() {
                    editor.error = None;
                }
                response.accessible_name(executable_label);
                if ui
                    .add_enabled(
                        editable,
                        egui::Button::new(text(language, "startup.browse")),
                    )
                    .accessible_name(message(
                        language,
                        "startup.browse_agent_executable_named",
                        &[("value1", &editor.agent_id)],
                    ))
                    .clicked()
                {
                    event = Some(Event::BrowseExecutable);
                }
            });
            let args_label = text(language, "startup.arguments_json_array");
            ui.label(args_label);
            let args = ui.add_enabled(
                editable,
                egui::TextEdit::multiline(&mut editor.args)
                    .desired_rows(5)
                    .desired_width(ui.available_width()),
            );
            if args.changed() {
                editor.error = None;
            }
            args.accessible_name(args_label);

            ui.columns(2, |columns| {
                let input_mode_label = text(language, "startup.input_mode");
                columns[0].label(input_mode_label);
                columns[0].add_enabled_ui(editable, |ui| {
                    egui::ComboBox::from_id_salt("native-agent-input-mode")
                        .selected_text(&editor.input_mode)
                        .width(ui.available_width())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut editor.input_mode, "stdin".into(), "stdin");
                            ui.selectable_value(&mut editor.input_mode, "file".into(), "file");
                        })
                        .response
                        .accessible_name(input_mode_label);
                });
                let sessions_label = text(language, "startup.profile_sessions");
                columns[1].label(sessions_label);
                columns[1]
                    .add_enabled(
                        editable,
                        egui::Slider::new(&mut editor.max_sessions, 1..=3).show_value(true),
                    )
                    .accessible_name(sessions_label);
            });
            let refresh_label = text(language, "startup.quota_refresh_minutes");
            ui.label(refresh_label);
            ui.add_enabled(
                editable,
                egui::Slider::new(&mut editor.quota_refresh_minutes, 1..=60).show_value(true),
            )
            .accessible_name(refresh_label);
            if let Some(next) = fallback::show(profiles, snapshot, editable, editor, language, ui) {
                event = Some(next);
            }
            (event, selected, id_changed)
        })
        .inner;

    ui.label(text(
        language,
        "startup.permissions_follow_from_the_cli_codex_runs_in",
    ));
    if let Some(template) = selected {
        let old_executable = editor.executable().to_owned();
        editor.switch_provider(&template, profiles);
        if old_executable != editor.executable() {
            event = Some(Event::ResolveExecutable(editor.executable().to_owned()));
        }
    }
    if id_changed {
        editor.error = None;
    }
    let duplicate = !editor.agent_id_is_unique(profiles);
    let parsed = serde_json::from_str::<Vec<String>>(&editor.args);
    let validation_error = if editor.agent_id.trim().is_empty() {
        Some(text(language, "startup.agent_id_required").to_owned())
    } else if duplicate {
        Some(text(language, "startup.agent_id_already_used").to_owned())
    } else if let Err(error) = &parsed {
        let error = error.to_string();
        Some(message(
            language,
            "startup.invalid_profile_arguments",
            &[("value1", &error)],
        ))
    } else {
        None
    };
    let fallback_error = fallback::validation_error(profiles, editor, language);
    if let Some(error) = validation_error {
        ui.colored_label(ui.visuals().error_fg_color, &error)
            .accessible_name(error);
    } else if let Some(error) = &fallback_error {
        ui.colored_label(ui.visuals().error_fg_color, error)
            .accessible_name(error.clone());
    } else if let Some(error) = &editor.error {
        ui.colored_label(ui.visuals().error_fg_color, error)
            .accessible_name(error.clone());
    }
    ui.horizontal(|ui| {
        let can_save = editable
            && parsed.is_ok()
            && !duplicate
            && !editor.agent_id.trim().is_empty()
            && fallback_error.is_none();
        let save = if can_save {
            egui::Button::new(
                egui::RichText::new(text(language, "startup.save_profile"))
                    .color(ui.visuals().widgets.active.fg_stroke.color),
            )
            .fill(ui.visuals().widgets.active.bg_fill)
        } else {
            egui::Button::new(text(language, "startup.save_profile"))
        };
        if ui.add_enabled(can_save, save).clicked() {
            event = Some(Event::SaveProfiles);
        }
        if ui
            .add_enabled(
                editable,
                egui::Button::new(text(language, "startup.cancel")),
            )
            .clicked()
        {
            editor.cancel();
        }
    });
    event
}
