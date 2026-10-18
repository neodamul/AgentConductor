use super::{Event, message, text};
use eframe::egui;
use serde_json::Value;

pub(super) fn show(
    profile: &Value,
    snapshot: &Value,
    language: &str,
    can_edit: bool,
    profile_editing: bool,
    saved: bool,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let id = profile["id"].as_str().unwrap_or_default();
    let parser = profile["parser"].as_str().unwrap_or("custom");
    if parser == "custom" {
        let label = text(
            language,
            "startup.configure_model_ids_cli_discovery_unavailable",
        );
        return ui
            .add_enabled(
                can_edit && saved && !profile_editing,
                egui::Button::new(text(language, "startup.models")),
            )
            .accessible_name(label)
            .clicked()
            .then(|| Event::ConfigureCatalog(id.to_owned()));
    }
    let busy = super::super::profile_tools::metadata_active(snapshot, "catalog", id);
    if ui
        .add_enabled(
            can_edit && saved && !busy,
            egui::Button::new(format!("{} ↻", text(language, "startup.models"))),
        )
        .accessible_name(message(language, "startup.refresh_models_for_agent", id))
        .clicked()
    {
        Some(Event::DiscoverModels(id.to_owned()))
    } else {
        None
    }
}
