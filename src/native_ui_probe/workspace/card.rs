use eframe::egui::{self, CornerRadius, Frame};

pub(crate) fn card(ui: &egui::Ui) -> Frame {
    Frame::new()
        .fill(ui.visuals().window_fill)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(24.0)
}
