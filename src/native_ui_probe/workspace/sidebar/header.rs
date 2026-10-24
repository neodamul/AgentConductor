use super::text;
use eframe::egui::{self, RichText, Ui};

pub(super) fn show(language: &str, can_add: bool, ui: &mut Ui) -> bool {
    let mut add = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                RichText::new(text(language, "startup.your_team"))
                    .small()
                    .strong()
                    .color(ui.visuals().weak_text_color()),
            );
            ui.heading(text(language, "startup.cli_agents"))
                .accessible_name(text(language, "startup.cli_agents"));
        });
        if ui
            .add_enabled(can_add, egui::Button::new(text(language, "startup.add")))
            .accessible_name(text(language, "startup.register_cli_agent"))
            .clicked()
        {
            add = true;
        }
    });
    add
}
