use eframe::egui;
use serde_json::Value;

pub fn show(
    rules: &mut crate::roles::RoleRules,
    profiles: &[Value],
    editable: bool,
    ui: &mut egui::Ui,
) {
    show_in_language(rules, profiles, editable, "en", ui);
}

pub fn show_in_language(
    rules: &mut crate::roles::RoleRules,
    profiles: &[Value],
    editable: bool,
    language: &str,
    ui: &mut egui::Ui,
) {
    let text = |id| crate::ui::messages::in_language(language, id);
    ui.separator();
    for role in crate::roles::ROLES {
        let role_name = role_name(language, role);
        let heading_color = ui.visuals().widgets.noninteractive.fg_stroke.color;
        let row_width = (ui.available_width() - 48.0).max(0.0);
        let agent_name = format!("{role_name} {}", text("startup.agent"));
        let model_word = if language == "ko" {
            text("startup.model")
        } else {
            "model"
        };
        let model_name = format!("{role_name} {model_word}");
        let entries = rules.entry(role.into()).or_default();
        let can_add = editable
            && entries.len() < 5
            && profiles.iter().any(|profile| {
                profile["id"].as_str().is_some_and(|id| {
                    !id.is_empty() && !entries.iter().any(|entry| entry.agent == id)
                })
            });
        let mut remove = None;
        let mut add = None;
        ui.group(|ui| {
            ui.set_width(row_width);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(role_name).strong().color(heading_color))
                    .accessible_name(role_name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_enabled_ui(can_add, |ui| {
                        ui.menu_button(text("startup.add_agent"), |ui| {
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
                            .accessible_name(agent_name.clone());
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
                                        entry
                                            .model
                                            .as_deref()
                                            .unwrap_or(text("startup.default_model")),
                                    )
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut entry.model,
                                            None,
                                            text("startup.default_model"),
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
                                    .accessible_name(model_name.clone());
                            }
                        }
                        if ui.button(text("startup.remove")).clicked() {
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

fn role_name(language: &str, role: &'static str) -> &'static str {
    if language != "ko" {
        return role;
    }
    crate::ui::messages::in_language(
        language,
        match role {
            "verification" => "startup.verification",
            "development" => "startup.development",
            "documentation" => "startup.documentation",
            "knowledge" => "startup.knowledge",
            "testing" => "startup.testing",
            _ => return role,
        },
    )
}
