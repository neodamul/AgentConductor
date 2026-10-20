use super::{message, text};
use crate::native_ui_probe::workspace::profile_tools::{Editor, Event};
use eframe::egui;
use serde_json::Value;

mod fallback;
mod fields;

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
            if ui.available_width() < 700.0 {
                selected = fields::provider(snapshot, editable, editor, language, ui);
                id_changed = fields::agent_id(editable, editor, language, ui);
            } else {
                ui.columns(2, |columns| {
                    selected =
                        fields::provider(snapshot, editable, editor, language, &mut columns[0]);
                    id_changed = fields::agent_id(editable, editor, language, &mut columns[1]);
                });
            }
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

            egui::CollapsingHeader::new(text(language, "startup.profile_rules_json"))
                .default_open(false)
                .show(ui, |ui| {
                    let extensions = ui.add_enabled(
                        editable,
                        egui::TextEdit::multiline(&mut editor.extensions)
                            .desired_rows(6)
                            .desired_width(ui.available_width()),
                    );
                    if extensions.changed() {
                        editor.error = None;
                    }
                    extensions.accessible_name(text(language, "startup.profile_rules_json"));
                });

            if ui.available_width() < 700.0 {
                fields::input_mode(editable, editor, language, ui);
                fields::sessions(editable, editor, language, ui);
            } else {
                ui.columns(2, |columns| {
                    fields::input_mode(editable, editor, language, &mut columns[0]);
                    fields::sessions(editable, editor, language, &mut columns[1]);
                });
            }
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
    let rules = serde_json::from_str::<Value>(&editor.extensions);
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
    } else if let Err(error) = &rules {
        Some(message(
            language,
            "startup.invalid_profile_rules",
            &[("value1", &error.to_string())],
        ))
    } else if !rules.as_ref().is_ok_and(Value::is_object) {
        Some(text(language, "startup.profile_rules_must_be_object").to_owned())
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
            && rules.as_ref().is_ok_and(Value::is_object)
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
