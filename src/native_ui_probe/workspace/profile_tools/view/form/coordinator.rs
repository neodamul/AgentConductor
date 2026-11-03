use crate::native_ui_probe::workspace::profile_tools::Editor;
use eframe::egui;

pub(super) fn show(editable: bool, editor: &mut Editor, language: &str, ui: &mut egui::Ui) {
    let model_label = text(language, "startup.coordinator_model");
    ui.label(model_label);
    let automatic = text(language, "startup.automatic_model");
    ui.add_enabled_ui(editable, |ui| {
        egui::ComboBox::from_id_salt("native-coordinator-model")
            .selected_text(editor.coordinator_model.as_deref().unwrap_or(automatic))
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut editor.coordinator_model, None, automatic);
                if let Some(models) = editor.base_profile["catalog"]["models"].as_array() {
                    for model in models.iter().filter_map(serde_json::Value::as_str) {
                        ui.selectable_value(
                            &mut editor.coordinator_model,
                            Some(model.to_owned()),
                            model,
                        );
                    }
                }
            })
            .response
            .accessible_name(model_label);
    });

    if editor.base_profile["parser"] == "custom" {
        let label = text(language, "startup.read_only_arguments_json_array");
        ui.label(label);
        ui.add_enabled(
            editable,
            egui::TextEdit::multiline(&mut editor.read_only_args)
                .desired_rows(3)
                .desired_width(ui.available_width())
                .hint_text("[\"chat\", \"--read-only\", \"{model}\", \"{task_file}\"]"),
        )
        .accessible_name(label);
    }
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}
