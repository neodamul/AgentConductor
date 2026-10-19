use super::{Probe, profile_from_template, profile_tools, show_role_rules};
use eframe::egui::{self, Color32, CornerRadius, Frame, RichText, Stroke};
use serde_json::Value;

pub(super) fn show(
    probe: &mut Probe,
    editable: bool,
    snapshot: &Value,
    ui: &mut egui::Ui,
) -> Option<profile_tools::Event> {
    let mut event = None;
    Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0, Color32::from_rgb(220, 220, 226)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(RichText::new("TEAM").small().strong());
            let mut selected = None;
            ui.add_enabled_ui(editable && probe.profiles.len() < 16, |ui| {
                ui.menu_button("Add agent", |ui| {
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
                            format!("Coordinator: {id}"),
                        );
                    });
                });
            }
            show_role_rules(&mut probe.roles, &probe.profiles, editable, ui);
            event = profile_tools::show(
                &mut probe.profiles,
                &mut probe.coordinator,
                &mut probe.roles,
                snapshot,
                editable,
                &mut probe.profile_editor,
                ui,
            );
        });
    event
}
