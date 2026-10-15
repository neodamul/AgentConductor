use eframe::egui;
use serde_json::Value;

pub fn show(
    rules: &mut crate::roles::RoleRules,
    profiles: &[Value],
    editable: bool,
    ui: &mut egui::Ui,
) {
    ui.separator();
    ui.label("Role assignments · first agent is preferred");
    for role in crate::roles::ROLES {
        let entries = rules.entry(role.into()).or_default();
        let mut remove = None;
        let mut add = None;
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(role);
                ui.add_enabled_ui(editable && entries.len() < 5, |ui| {
                    ui.menu_button("Add agent", |ui| {
                        for profile in profiles {
                            let id = profile["id"].as_str().unwrap_or_default();
                            if !id.is_empty()
                                && !entries.iter().any(|entry| entry.agent == id)
                                && ui.button(id).clicked()
                            {
                                add = Some(id.to_owned());
                                ui.close();
                            }
                        }
                    });
                });
            });
            for (index, entry) in entries.iter_mut().enumerate() {
                ui.add_enabled_ui(editable, |ui| {
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_id_salt((role, index, "agent"))
                            .selected_text(&entry.agent)
                            .show_ui(ui, |ui| {
                                for profile in profiles {
                                    if let Some(id) = profile["id"].as_str() {
                                        ui.selectable_value(&mut entry.agent, id.into(), id);
                                    }
                                }
                            })
                            .response
                            .accessible_name(format!("{role} agent"));
                        if let Some(profile) = profiles.iter().find(|p| p["id"] == entry.agent) {
                            let models = profile["catalog"]["models"]
                                .as_array()
                                .into_iter()
                                .flatten()
                                .filter_map(Value::as_str)
                                .collect::<Vec<_>>();
                            if !models.is_empty() {
                                egui::ComboBox::from_id_salt((role, index, "model"))
                                    .selected_text(
                                        entry.model.as_deref().unwrap_or("Default model"),
                                    )
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut entry.model,
                                            None,
                                            "Default model",
                                        );
                                        for model in models {
                                            ui.selectable_value(
                                                &mut entry.model,
                                                Some(model.to_owned()),
                                                model,
                                            );
                                        }
                                    })
                                    .response
                                    .accessible_name(format!("{role} model"));
                            }
                        }
                        if ui.button("Remove").clicked() {
                            remove = Some(index);
                        }
                    });
                });
            }
        });
        if let Some(index) = remove {
            entries.remove(index);
        }
        if let Some(agent) = add {
            entries.push(crate::roles::RoleAgent { agent, model: None });
        }
    }
}
