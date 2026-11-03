#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe as render;
use agentconductor::native_ui_probe::room;
use eframe::egui;
use serde_json::json;

#[test]
fn denial_uses_a_localized_error_label_and_failed_colors() {
    for (language, expected) in [("en", "Permission denied"), ("ko", "권한 거부")] {
        assert_eq!(
            render::status_label("permission_denied", language),
            expected
        );
        for dark in [false, true] {
            assert_eq!(
                render::status_colors("permission_denied", dark),
                render::status_colors("failed", dark)
            );
        }
        assert_eq!(
            render::status_color("permission_denied", 0.0),
            render::status_color("failed", 0.0)
        );
    }
}

#[test]
fn blocked_room_displays_denied_task_and_its_error_without_a_run_error() {
    for (language, expected) in [("en", "Permission denied"), ("ko", "권한 거부")] {
        let snapshot = json!({
            "job":{"state":"completed"},
            "report":{
                "state":"blocked",
                "plan":{"goal":"Inspect denial", "agents":[{"id":"local","parser":"custom"}],
                    "tasks":[{"id":"denied-task","agent":"local"}]},
                "tasks":[{"id":"denied-task","status":"permission_denied",
                    "error":"permission denied: tool bash: policy rejected"}]
            }
        });
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut state = room::State::default();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1280.0, 892.0),
                )),
                ..Default::default()
            },
            |ui| {
                room::show(&mut state, Some(&snapshot), false, None, language, ui);
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
        let visible = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) => Some(text.galley.job.text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ");
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        assert!(
            labels.iter().any(|label| label.contains(expected)),
            "{labels:?}"
        );
        assert!(
            visible.contains("permission denied: tool bash: policy rejected"),
            "{visible}"
        );
    }
}
