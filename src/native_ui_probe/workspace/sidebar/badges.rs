use eframe::egui::{self, CornerRadius, Frame, RichText};

pub(super) fn show(ui: &mut egui::Ui, label: &str) {
    let color = ui.visuals().hyperlink_color;
    Frame::new()
        .fill(color.gamma_multiply(0.12))
        .corner_radius(CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.add(egui::Label::new(RichText::new(label).small().color(color)).truncate())
                .on_hover_text(label)
        })
        .inner
        .accessible_name(label.to_owned());
}
