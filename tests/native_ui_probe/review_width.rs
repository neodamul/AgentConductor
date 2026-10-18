use super::{Probe, show};
use eframe::egui;
use serde_json::json;

#[test]
fn empty_plan_review_card_uses_the_available_content_width() {
    let context = egui::Context::default();
    let mut probe = Probe {
        snapshot: Some(json!({"proposal":{"plan":null}})),
        ..Default::default()
    };
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(860.0, 720.0),
            )),
            ..Default::default()
        },
        |ui| show(&mut probe, ui),
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let card_width = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.stroke.width > 0.0 => Some(rect.rect.width()),
            _ => None,
        })
        .max_by(f32::total_cmp)
        .unwrap_or_default();
    assert!(card_width >= 800.0, "Review card width was {card_width}px");
}
