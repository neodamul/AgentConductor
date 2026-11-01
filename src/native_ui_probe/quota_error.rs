use eframe::egui::{self, RichText};
use serde_json::Value;

pub(crate) fn show(snapshot: &Value, agent: Option<&str>, language: &str, ui: &mut egui::Ui) {
    let unavailable = crate::ui::messages::in_language(language, "room.quota_unavailable");
    for (id, quota) in snapshot["quotas"].as_object().into_iter().flatten() {
        if agent.is_some_and(|agent| agent != id) {
            continue;
        }
        if let Some(error) = quota["error"]
            .as_str()
            .filter(|error| !error.trim().is_empty())
        {
            let diagnostic = format!("{id} · {unavailable}: {error}");
            ui.label(
                RichText::new(&diagnostic)
                    .small()
                    .color(ui.visuals().error_fg_color),
            )
            .accessible_name(diagnostic)
            .on_hover_text(error);
        }
    }
}
