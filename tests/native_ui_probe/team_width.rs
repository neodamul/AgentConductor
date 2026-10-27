use super::*;
use eframe::egui;

#[test]
fn role_card_fills_the_available_content_width() {
    let width = 835.0;
    let context = egui::Context::default();
    let mut probe = Probe::default();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 600.0),
            )),
            ..Default::default()
        },
        |ui| {
            super::show(&mut probe, false, &serde_json::Value::Null, ui);
        },
    );
    let card_width = output
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Rect(shape) if shape.rect.height() > 100.0 => Some(shape.rect.width()),
            _ => None,
        })
        .reduce(f32::max)
        .unwrap_or_default();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        card_width >= width * 0.95,
        "Agent Roles card should fill the main content area: {card_width}px of {width}px"
    );
}
