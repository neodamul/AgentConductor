use agentconductor::native_ui_probe::{Probe, status_color, status_colors, status_label};
use eframe::egui::{self, Color32};

#[test]
fn native_status_labels_are_localized_and_accessible() {
    let snapshot = serde_json::json!({
        "report": {
            "state": "running",
            "plan": {"goal": "테스트", "tasks": [{"id": "구현", "agent": "claude"}]},
            "tasks": [{"id": "구현", "status": "running"}]
        }
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        Probe::render_snapshot_in_language(&snapshot, "ko", ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    let visible = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(labels.contains(&"작업 구현 상태: 실행 중"));
    assert!(visible.contains("실행 중"));
    assert_eq!(status_label("unknown", "ko"), "unknown");
}

#[test]
fn native_status_tags_keep_semantic_color_contrast_in_both_themes() {
    for status in [
        "running",
        "completed",
        "pending",
        "waiting_approval",
        "blocked",
        "failed",
        "stopped",
    ] {
        for dark in [false, true] {
            let (foreground, background) = status_colors(status, dark);
            assert!(
                contrast(foreground, background) >= 4.5,
                "{status}, dark={dark}"
            );
        }
    }
    assert_ne!(
        status_colors("running", false),
        status_colors("blocked", false)
    );
    assert_ne!(status_color("running", 0.0), status_color("running", 0.5));
    assert_eq!(status_label("blocked", "ko"), "막힘");
}

fn contrast(foreground: Color32, background: Color32) -> f64 {
    fn luminance(color: Color32) -> f64 {
        let channels = color.to_array()[..3]
            .iter()
            .map(|channel| {
                let value = f64::from(*channel) / 255.0;
                if value <= 0.04045 {
                    value / 12.92
                } else {
                    ((value + 0.055) / 1.055).powf(2.4)
                }
            })
            .collect::<Vec<_>>();
        0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]
    }
    let (light, dark) = (luminance(foreground), luminance(background));
    (light.max(dark) + 0.05) / (light.min(dark) + 0.05)
}
