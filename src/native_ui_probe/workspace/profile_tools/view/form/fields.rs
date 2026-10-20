use crate::native_ui_probe::workspace::profile_tools::Editor;
use eframe::egui;
use serde_json::Value;

pub(super) fn provider(
    snapshot: &Value,
    editable: bool,
    editor: &Editor,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Value> {
    let label = crate::ui::messages::in_language(language, "startup.provider");
    ui.label(label);
    let mut selected = None;
    ui.add_enabled_ui(editable, |ui| {
        egui::ComboBox::from_id_salt("native-agent-provider")
            .selected_text(template_name(snapshot, &editor.base_profile))
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                if let Some(templates) = snapshot["templates"].as_array() {
                    for template in templates {
                        let parser = template["parser"].as_str().unwrap_or("custom");
                        if ui
                            .selectable_label(
                                same_template(&editor.base_profile, template),
                                template["id"].as_str().unwrap_or(parser),
                            )
                            .clicked()
                        {
                            selected = Some(template.clone());
                        }
                    }
                }
                let registered = snapshot["templates"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|template| same_template(&editor.base_profile, template));
                if ui
                    .selectable_label(
                        editor.base_profile["parser"] == "custom" && !registered,
                        "custom",
                    )
                    .clicked()
                {
                    selected = Some(super::super::super::custom_template());
                }
            })
            .response
            .accessible_name(label);
    });
    selected
}

pub(super) fn agent_id(
    editable: bool,
    editor: &mut Editor,
    language: &str,
    ui: &mut egui::Ui,
) -> bool {
    let label = crate::ui::messages::in_language(language, "startup.agent_id");
    ui.label(label);
    ui.add_enabled_ui(editable, |ui| {
        ui.add(egui::TextEdit::singleline(&mut editor.agent_id))
            .accessible_name(label)
            .changed()
    })
    .inner
}

pub(super) fn input_mode(editable: bool, editor: &mut Editor, language: &str, ui: &mut egui::Ui) {
    let label = crate::ui::messages::in_language(language, "startup.input_mode");
    ui.label(label);
    ui.add_enabled_ui(editable, |ui| {
        egui::ComboBox::from_id_salt("native-agent-input-mode")
            .selected_text(&editor.input_mode)
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut editor.input_mode, "stdin".into(), "stdin");
                ui.selectable_value(&mut editor.input_mode, "file".into(), "file");
            })
            .response
            .accessible_name(label);
    });
}

pub(super) fn sessions(editable: bool, editor: &mut Editor, language: &str, ui: &mut egui::Ui) {
    let label = crate::ui::messages::in_language(language, "startup.profile_sessions");
    ui.label(label);
    ui.add_enabled(
        editable,
        egui::Slider::new(&mut editor.max_sessions, 1..=3).show_value(true),
    )
    .accessible_name(label);
}

fn template_name<'a>(snapshot: &'a Value, profile: &'a Value) -> &'a str {
    snapshot["templates"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|template| same_template(profile, template))
        .and_then(|template| template["id"].as_str())
        .unwrap_or_else(|| profile["parser"].as_str().unwrap_or("custom"))
}

fn same_template(profile: &Value, template: &Value) -> bool {
    profile["parser"] == template["parser"]
        && profile["executable"] == template["executable"]
        && profile["args"] == template["args"]
}
