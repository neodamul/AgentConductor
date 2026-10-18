#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::header;
use eframe::egui::{self, Shape};

#[test]
fn workspace_header_matches_the_compact_legacy_step_row() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(860.0, 220.0),
            )),
            ..Default::default()
        },
        |ui| {
            header::show(0, "en", ui);
        },
    );
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    let has_large_badge = output.shapes.iter().any(
        |shape| matches!(&shape.shape, Shape::Circle(circle) if (circle.radius - 11.0).abs() < 0.1),
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    for label in [
        "1. Workspace settings",
        "2. Agent roles",
        "3. Review the plan",
    ] {
        assert!(
            labels.contains(&label.to_owned()),
            "missing compact step label {label:?}"
        );
    }
    assert!(
        !labels.contains(&"Prepared".to_owned()),
        "status belongs in the agent list"
    );
    assert!(
        !has_large_badge,
        "legacy steps use compact selected labels, not number badges"
    );
}
