use eframe::egui::{self, Color32, RichText, Stroke};

pub(super) fn show(message: &str, ui: &mut egui::Ui) {
    let warning = ui.visuals().warn_fg_color;
    egui::Frame::new()
        .fill(panel_fill(warning))
        .stroke(Stroke::new(1.0, warning.gamma_multiply(0.55)))
        .inner_margin(egui::Margin::same(9))
        .corner_radius(egui::CornerRadius::same(5))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("⚠").strong().color(warning));
                ui.add(egui::Label::new(RichText::new(message).color(warning).size(14.0)).wrap())
                    .accessible_name(message);
            });
        });
}

pub(super) fn panel_fill(warning: Color32) -> Color32 {
    Color32::from_rgba_unmultiplied(warning.r(), warning.g(), warning.b(), 28)
}
