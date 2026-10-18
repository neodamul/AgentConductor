use super::{Probe, card, profile_tools, show_role_rules_in_language, sidebar};
use eframe::egui;
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
            egui::RichText::new(text(language, "startup.step_2_team"))
                .small()
                .strong()
                .color(ui.visuals().hyperlink_color),
        );
        ui.heading(
            egui::RichText::new(text(language, "startup.agent_roles"))
                .strong()
                .color(ui.visuals().text_color()),
        );
        ui.label(
            egui::RichText::new(text(
                language,
                "startup.role_assignments_first_agent_preferred",
            ))
            .small()
            .color(ui.visuals().weak_text_color()),
        );
        ui.horizontal(|ui| {
            if ui
                .button(text(language, "startup.suggest_defaults"))
                .clicked()
                && editable
            {
                probe.roles = super::suggested_role_rules(&probe.profiles);
            }
        });
        if probe.profile_saved {
            ui.label(text(language, "startup.agent_list_saved"));
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
                show_registry: false,
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

#[cfg(test)]
#[path = "../../../tests/native_ui_probe/registration.rs"]
mod tests;

pub(super) fn handle_event(probe: &mut Probe, event: sidebar::Event) {
    match event {
        sidebar::Event::AddAgent => {
            if probe.profiles.len() >= 16 || probe.profile_editor.is_editing() {
                return;
            }
            let template = profile_tools::custom_template();
            if let Some(profile) = profile_from_template(&template, &probe.profiles) {
                probe.profile_editor.open_new(&profile);
                probe.profile_saved = false;
                probe.workspace_step = 1;
                if probe.worker.is_some() {
                    probe.resolve_executable(probe.profile_editor.executable().to_owned());
                }
            }
        }
        sidebar::Event::RemoveAgent(id) => {
            probe.profile_editor.request_remove(&id);
            probe.workspace_step = 1;
        }
        sidebar::Event::DiscoverModels(id) => {
            probe.request_metadata(profile_tools::Event::Discover(id));
        }
        sidebar::Event::EditAgent(id) => {
            if let Some(profile) = probe.profiles.iter().find(|item| item["id"] == id) {
                probe.profile_editor.open(profile);
                probe.workspace_step = 1;
            }
        }
        sidebar::Event::Quota(id) => {
            probe.request_metadata(profile_tools::Event::Quota(id));
        }
        sidebar::Event::MoveAgent(id, target)
            if profile_tools::reorder_profiles(&mut probe.profiles, &id, target) =>
        {
            probe.send_profile_save();
        }
        sidebar::Event::MoveAgent(..) => {}
    }
}
