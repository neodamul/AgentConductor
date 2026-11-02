use super::Event;
use eframe::egui::{self, RichText};
use serde_json::Value;

pub(super) fn show(
    agent: &str,
    snapshot: &Value,
    language: &str,
    can_refresh: bool,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let badge = &snapshot["quota_badges"][agent];
    let quota = badge["text"].as_str().unwrap_or("—");
    let title = badge["title"].as_str().unwrap_or_default();
    let mut action = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        ui.label(
            RichText::new(quota)
                .small()
                .strong()
                .monospace()
                .color(ui.visuals().text_color()),
        )
        .on_hover_text(title)
        .accessible_name(if title.is_empty() {
            quota.to_owned()
        } else {
            format!("{quota}: {title}")
        });
        if super::super::profile_tools::metadata_active(snapshot, "quota", agent) {
            ui.spinner();
        } else {
            let refresh = crate::ui::messages::interpolate(
                crate::ui::messages::in_language(language, "startup.refresh_quota"),
                &[("value1", agent)],
            );
            if ui
                .add_enabled(can_refresh, egui::Button::new("↻").small())
                .accessible_name(refresh.clone())
                .on_hover_text(refresh)
                .clicked()
            {
                action = Some(Event::Quota(agent.to_owned()));
            }
        }
    });
    crate::native_ui_probe::quota_error::show(snapshot, Some(agent), language, ui);
    action
}
