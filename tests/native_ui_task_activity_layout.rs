#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::task_activity::{self, Tab};
use eframe::egui;
use serde_json::json;

#[test]
fn activity_entries_fill_the_review_column() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let plan = json!({"id":"one","prompt":"Implement the refresh guard"});
    let details = json!({"task":{"id":"one"},"events":[]});
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(620.0, 500.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                task_activity::show(
                    &mut Tab::Activity,
                    "custom",
                    &plan,
                    Some(&details),
                    None,
                    "en",
                    ui,
                );
            });
        },
    );
    let group_width = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.stroke.width > 0.0 => Some(rect.rect.width()),
            _ => None,
        })
        .unwrap_or_default();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        group_width >= 540.0,
        "activity entry should fill the review column, got {} px",
        group_width
    );
}
