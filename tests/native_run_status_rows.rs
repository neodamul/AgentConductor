#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::Probe;
use eframe::egui::{self, Color32, Shape};
use serde_json::json;

fn row_outline(status: &str, time: f64, reduced_motion: bool) -> Option<(f32, Color32)> {
    let snapshot = json!({
        "report":{
            "state":"running",
            "plan":{"goal":"Inspect task state","tasks":[{"id":"task-1","agent":"codex"}]},
            "tasks":[{"id":"task-1","status":status}]
        }
    });
    let context = egui::Context::default();
    context.set_visuals(egui::Visuals::light());
    context.global_style_mut(|style| {
        if reduced_motion {
            style.animation_time = 0.0;
        }
    });
    let mut output = context.run_ui(
        egui::RawInput {
            time: Some(time),
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(860.0, 720.0),
            )),
            ..Default::default()
        },
        |ui| Probe::render_snapshot(&snapshot, ui),
    );
    let result = output.shapes.iter().find_map(|shape| match &shape.shape {
        Shape::Rect(rect) if rect.rect.width() > 300.0 && rect.stroke.width > 0.0 => {
            Some((rect.stroke.width, rect.stroke.color))
        }
        _ => None,
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    result
}

#[test]
fn failed_task_has_a_red_row_border_and_tool_wait_border_pulses() {
    let failed = row_outline("failed", 0.1, false).expect("failed task row outline");
    assert_eq!(failed.1, Color32::from_rgb(196, 49, 42));

    let on = row_outline("waiting_tool_approval", 0.1, false).expect("waiting task outline");
    let off = row_outline("waiting_tool_approval", 0.8, false).expect("waiting task outline");
    assert_ne!(on, off, "waiting-for-tool row border should pulse");

    let reduced_on = row_outline("waiting_tool_approval", 0.1, true).unwrap();
    let reduced_off = row_outline("waiting_tool_approval", 0.8, true).unwrap();
    assert_eq!(
        reduced_on, reduced_off,
        "reduced motion keeps a static border"
    );
}
