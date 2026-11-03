use super::super::Probe;
use eframe::egui::{self, Align, Layout, vec2};
use serde_json::Value;

const FIELD_HEIGHT: f32 = 70.0;

pub(super) fn show(probe: &mut Probe, can_edit: bool, ui: &mut egui::Ui) {
    let locale = &probe.navigation.appearance;
    let name = locale.text("startup.workspace_name");
    let name_hint = locale.text("startup.workspace_name_hint");
    let coordinator_label = locale.text("startup.planning_coordinator");
    let coordinator_none = locale.text("startup.coordinator_none");
    let language = locale.language().to_owned();
    let coordinator = probe.coordinator.clone();
    let selected = probe
        .profiles
        .iter()
        .find(|profile| profile["id"] == probe.coordinator)
        .and_then(|profile| profile["id"].as_str())
        .unwrap_or(coordinator_none)
        .to_owned();

    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = 40.0;
        let available = ui.available_width();
        let gap = 12.0;
        ui.spacing_mut().item_spacing.x = gap;
        if available < 700.0 {
            field(ui, available, |ui| {
                name_field(ui, name, name_hint, can_edit, probe)
            });
            let columns = (ui.available_width() - gap).max(0.0);
            ui.horizontal(|ui| {
                let coordinator_width = columns * 0.54;
                field(ui, coordinator_width, |ui| {
                    coordinator_field(
                        ui,
                        &mut probe.coordinator,
                        &probe.profiles,
                        &selected,
                        coordinator_label,
                        can_edit,
                    )
                });
                if let Some(profile) = probe
                    .profiles
                    .iter_mut()
                    .find(|profile| profile["id"] == coordinator)
                {
                    field(ui, columns - coordinator_width, |ui| {
                        coordinator_model_selector(profile, &language, can_edit, ui)
                    });
                }
            });
        } else {
            let row_width = (available - 35.0).max(0.0);
            let columns = (row_width - 2.0 * gap).max(0.0);
            let name_width = columns * 0.475;
            let coordinator_width = columns * 0.2625;
            let model_width = columns - name_width - coordinator_width;
            ui.horizontal(|ui| {
                field(ui, name_width, |ui| {
                    name_field(ui, name, name_hint, can_edit, probe)
                });
                field(ui, coordinator_width, |ui| {
                    coordinator_field(
                        ui,
                        &mut probe.coordinator,
                        &probe.profiles,
                        &selected,
                        coordinator_label,
                        can_edit,
                    )
                });
                if let Some(profile) = probe
                    .profiles
                    .iter_mut()
                    .find(|profile| profile["id"] == coordinator)
                {
                    field(ui, model_width, |ui| {
                        coordinator_model_selector(profile, &language, can_edit, ui)
                    });
                }
            });
        }
    });
}

fn field(ui: &mut egui::Ui, width: f32, show: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(
        vec2(width, FIELD_HEIGHT),
        Layout::top_down(Align::Min),
        show,
    );
}

fn name_field(ui: &mut egui::Ui, name: &str, hint: &str, can_edit: bool, probe: &mut Probe) {
    ui.label(name);
    ui.add_enabled(
        can_edit,
        egui::TextEdit::singleline(&mut probe.name)
            .desired_width(ui.available_width())
            .margin(vec2(8.0, 6.0))
            .hint_text(hint),
    )
    .accessible_name(name);
}

fn coordinator_field(
    ui: &mut egui::Ui,
    coordinator: &mut String,
    profiles: &[Value],
    selected: &str,
    label: &str,
    can_edit: bool,
) {
    ui.label(label);
    ui.add_enabled_ui(can_edit && !profiles.is_empty(), |ui| {
        egui::ComboBox::from_id_salt("native-workspace-coordinator")
            .selected_text(selected)
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for profile in profiles {
                    if let Some(id) = profile["id"].as_str() {
                        ui.selectable_value(coordinator, id.into(), id);
                    }
                }
            })
            .response
            .accessible_name(label);
    });
}

fn coordinator_model_selector(
    profile: &mut Value,
    language: &str,
    can_edit: bool,
    ui: &mut egui::Ui,
) {
    let model_label = crate::ui::messages::in_language(language, "startup.coordinator_model");
    let cli_default = crate::ui::messages::in_language(language, "startup.cli_default");
    let models = profile["catalog"]["models"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    let mut selected = profile["coordinator_model"]
        .as_str()
        .filter(|selected| models.iter().any(|model| model.as_str() == Some(selected)))
        .unwrap_or_default()
        .to_owned();
    let previous = selected.clone();
    ui.vertical(|ui| {
        let label = ui.label(model_label);
        ui.add_enabled_ui(can_edit && !models.is_empty(), |ui| {
            egui::ComboBox::from_id_salt("native-coordinator-model-choice")
                .selected_text(if selected.is_empty() {
                    cli_default
                } else {
                    &selected
                })
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected, String::new(), cli_default);
                    for model in models {
                        if let Some(model) = model.as_str() {
                            ui.selectable_value(&mut selected, model.to_owned(), model);
                        }
                    }
                })
                .response
                .labelled_by(label.id)
                .accessible_name(model_label);
        });
    });
    if selected != previous
        && let Some(profile) = profile.as_object_mut()
    {
        if selected.is_empty() {
            profile.remove("coordinator_model");
        } else {
            profile.insert("coordinator_model".into(), selected.into());
        }
    }
}
