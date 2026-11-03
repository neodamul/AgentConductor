use eframe::egui;
use serde_json::json;

#[test]
fn local_workspace_path_is_pinned_to_the_window_bottom() {
    let context = egui::Context::default();
    let mut probe = crate::native_ui_probe::Probe {
        snapshot: Some(json!({
            "root":"/private/tmp/ac-demo/desktop",
            "config":{"profiles":[],"goal":"","repository":""}
        })),
        ..Default::default()
    };
    let expected = format!(
        "{}{}",
        crate::ui::messages::in_language(
            probe.navigation.appearance.language(),
            "startup.local_workspace"
        ),
        "/private/tmp/ac-demo/desktop"
    );
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 892.0),
            )),
            ..Default::default()
        },
        |ui| super::show(&mut probe, ui),
    );
    let status = output.shapes.iter().find_map(|shape| match &shape.shape {
        egui::Shape::Text(text) if text.galley.job.text == expected => Some(text.pos.y),
        _ => None,
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        status.is_some_and(|y| y > 840.0),
        "status should remain at the viewport bottom: {status:?}"
    );
}

#[test]
fn loading_status_remains_visible_before_the_first_snapshot() {
    let context = egui::Context::default();
    let mut probe = crate::native_ui_probe::Probe::default();
    let expected = crate::ui::messages::in_language(
        probe.navigation.appearance.language(),
        "startup.reading_local_workspace",
    );
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 892.0),
            )),
            ..Default::default()
        },
        |ui| super::show(&mut probe, ui),
    );
    assert!(output.shapes.iter().any(|shape| matches!(
        &shape.shape,
        egui::Shape::Text(text) if text.galley.job.text == expected
    )));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
