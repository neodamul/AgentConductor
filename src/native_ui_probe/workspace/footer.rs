use eframe::egui::{self, Frame, RichText};
use serde_json::Value;

pub(super) fn show(snapshot: &Value, language: &str, ui: &mut egui::Ui) {
    let label = snapshot["root"].as_str().map_or_else(
        || crate::ui::messages::in_language(language, "startup.reading_local_workspace").into(),
        |root| {
            format!(
                "{}{}",
                crate::ui::messages::in_language(language, "startup.local_workspace"),
                root
            )
        },
    );
    egui::Panel::bottom("native-workspace-status")
        .frame(
            Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .inner_margin(egui::Margin::symmetric(14, 5)),
        )
        .show(ui, |ui| {
            ui.label(
                RichText::new(label)
                    .small()
                    .color(ui.visuals().weak_text_color()),
            );
        });
}
