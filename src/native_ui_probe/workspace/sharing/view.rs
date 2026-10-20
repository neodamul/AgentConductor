use super::{Event, ResultMessage, State};
use eframe::egui;
use serde_json::Value;

pub fn show(
    state: &mut State,
    language: &str,
    enabled: bool,
    dirty: bool,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let mut event = None;
    let text = |id| crate::ui::messages::in_language(language, id);
    let actions_enabled = enabled && !dirty;
    let heading_color = ui.visuals().widgets.noninteractive.fg_stroke.color;
    ui.group(|ui| {
        let heading = text("startup.workspace_settings_sharing");
        ui.label(egui::RichText::new(heading).strong().color(heading_color))
            .accessible_name(heading);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    actions_enabled,
                    egui::Button::new(text("startup.export_settings_button")),
                )
                .accessible_name(text("startup.export_workspace_settings_accessible"))
                .clicked()
            {
                event = Some(Event::Export);
            }
            if ui
                .add_enabled(
                    actions_enabled,
                    egui::Button::new(text("startup.import_settings_button")),
                )
                .accessible_name(text("startup.import_workspace_settings_accessible"))
                .clicked()
            {
                event = Some(Event::Import);
            }
            if ui
                .button(text("startup.close"))
                .accessible_name(text("startup.close"))
                .clicked()
            {
                state.open = false;
            }
        });
        if dirty && enabled {
            let warning = text("startup.sharing_unsaved_changes");
            ui.label(warning).accessible_name(warning);
        }
        if let Some(preview) = &state.preview {
            let prompt = message(
                language,
                "startup.import_from_folder",
                &[("value1", &state.folder)],
            );
            let accessible = message(
                language,
                "startup.import_workspace_from_folder",
                &[("value1", &state.folder)],
            );
            ui.label(prompt).accessible_name(accessible);
            let settings = &preview["settings"];
            let agent_count = settings["agents"]
                .as_array()
                .map_or(0, Vec::len)
                .to_string();
            let summary = message(
                language,
                "startup.workspace_sharing_summary",
                &[
                    (
                        "value1",
                        settings["name"]
                            .as_str()
                            .unwrap_or(text("startup.workspace_name_fallback")),
                    ),
                    ("value2", &agent_count),
                    (
                        "value3",
                        settings["coordinator"]
                            .as_str()
                            .unwrap_or(text("startup.coordinator_none")),
                    ),
                ],
            );
            ui.label(&summary).accessible_name(summary);
            if let Some(warnings) = preview["warnings"].as_array() {
                for warning in warnings.iter().filter_map(Value::as_str) {
                    ui.colored_label(egui::Color32::DARK_RED, warning)
                        .accessible_name(warning);
                }
            }
            ui.checkbox(
                &mut state.keep_verified,
                text("startup.keep_verified_claims"),
            )
            .accessible_name(text("startup.keep_verified_claims"));
            ui.horizontal(|ui| {
                if ui
                    .button(text("startup.import_and_replace_settings"))
                    .accessible_name(text("startup.import_and_replace_settings"))
                    .clicked()
                {
                    event = Some(Event::ConfirmImport);
                }
                if ui
                    .button(text("startup.cancel_import"))
                    .accessible_name(text("startup.cancel_import"))
                    .clicked()
                {
                    event = Some(Event::CancelImport);
                }
            });
        }
        if let Some(result) = &state.result {
            let result = match result {
                ResultMessage::Imported => text("startup.workspace_settings_imported").into(),
                ResultMessage::Exported(path) => message(
                    language,
                    "startup.workspace_settings_exported_to",
                    &[(
                        "value1",
                        if path.is_empty() {
                            text("startup.selected_folder")
                        } else {
                            path
                        },
                    )],
                ),
            };
            ui.label(&result).accessible_name(result);
        }
    });
    event
}

fn message(language: &str, id: &'static str, params: &[(&str, &str)]) -> String {
    crate::ui::messages::interpolate(crate::ui::messages::in_language(language, id), params)
}
