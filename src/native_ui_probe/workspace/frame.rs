use eframe::egui::{self, Frame};

pub(crate) fn root_frame(background: egui::Color32) -> Frame {
    Frame::new().fill(background).inner_margin(egui::Margin {
        left: 14,
        right: 28,
        top: 24,
        bottom: 24,
    })
}
