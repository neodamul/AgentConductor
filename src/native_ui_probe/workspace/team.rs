use super::{Probe, card, profile_tools, show_role_rules_in_language};
use eframe::egui::{self, RichText};
use serde_json::Value;

pub(super) fn show(
    probe: &mut Probe,
    editable: bool,
    snapshot: &Value,
    ui: &mut egui::Ui,
) -> Option<profile_tools::Event> {
    let mut event = None;
    let language = probe.navigation.appearance.language();
    card(ui).show(ui, |ui| {
        ui.label(
            RichText::new(text(language, "startup.your_team"))
                .small()
                .strong(),
        );
        let mut selected = None;
        ui.add_enabled_ui(editable && probe.profiles.len() < 16, |ui| {
            ui.menu_button(text(language, "startup.add_agent"), |ui| {
                if let Some(templates) = snapshot["templates"].as_array() {
                    for template in templates {
                        let id = template["id"].as_str().unwrap_or("agent");
                        let parser = template["parser"].as_str().unwrap_or("custom");
                        if ui.button(format!("{id} · {parser}")).clicked() {
                            selected = Some(template.clone());
                            ui.close();
                        }
                    }
                }
            });
        });
        if let Some(template) = selected
            && let Some(profile) = profile_from_template(&template, &probe.profiles)
        {
            let id = profile["id"].as_str().unwrap_or_default().to_owned();
            if probe.coordinator.is_empty() {
                probe.coordinator = id;
            }
            probe.profiles.push(profile);
        }
        for profile in &probe.profiles {
            let id = profile["id"].as_str().unwrap_or("unknown").to_owned();
            let parser = profile["parser"].as_str().unwrap_or("custom");
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{id} · {parser}")).strong());
                ui.add_enabled_ui(editable, |ui| {
                    ui.radio_value(
                        &mut probe.coordinator,
                        id.clone(),
                        crate::ui::messages::interpolate(
                            text(language, "startup.coordinator_named"),
                            &[("value1", &id)],
                        ),
                    );
                });
            });
        }
        show_role_rules_in_language(
            &mut probe.roles,
            &probe.profiles,
            editable,
            probe.navigation.appearance.language(),
            ui,
        );
        event = profile_tools::show_in_language(
            &mut probe.profiles,
            &mut probe.coordinator,
            &mut probe.roles,
            &mut probe.profile_editor,
            profile_tools::RenderContext {
                snapshot,
                editable,
                language,
            },
            ui,
        );
    });
    event
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

pub fn profile_from_template(template: &Value, profiles: &[Value]) -> Option<Value> {
    if profiles.len() >= 16 {
        return None;
    }
    let base = template["id"].as_str()?;
    let mut id = base.to_owned();
    let mut suffix = 2;
    while profiles.iter().any(|profile| profile["id"] == id) {
        id = format!("{base}-{suffix}");
        suffix += 1;
    }
    let mut profile = template.clone();
    profile["id"] = id.into();
    Some(profile)
}
