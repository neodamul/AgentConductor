#![cfg(feature = "native-ui-probe")]
//! The room carries run context without repeating shared application chrome.

use agentconductor::native_ui_probe::room::{self, State};
use eframe::egui::{self, Rect, epaint::Shape};

#[test]
fn the_room_context_does_not_repeat_the_shared_app_title() {
    let snapshot = serde_json::json!({
        "report": {
            "state": "running", "repo": "/tmp/project", "branch": "main",
            "plan": {"goal": "Rotate tokens"}
        },
        "phase": {
            "phase": "tokens", "slug": "auth", "description": "Rotate tokens",
            "phases": [{"id": "tokens", "name": "Authentication"}]
        }
    });
    let context = egui::Context::default();
    let mut state = State::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let texts: Vec<String> = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect();
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    assert!(
        texts.iter().all(|text| text != "Agent Room"),
        "the room repeats its native window title in the page: {texts:?}"
    );
    assert!(
        texts
            .iter()
            .all(|text| { text != "AC" && text != "AgentConductor" && text != &version }),
        "the shared shell already renders the application name and version: {texts:?}"
    );
    assert!(texts.iter().any(|text| text == "project"), "{texts:?}");
    assert!(
        texts.iter().any(|text| text.contains("Authentication")),
        "the room header omits the linked phase: {texts:?}"
    );
    assert!(
        texts
            .iter()
            .any(|text| text == "Local · CLI authentication"),
        "{texts:?}"
    );
}

#[test]
fn the_room_context_header_fits_both_locales_at_supported_widths() {
    let snapshot = serde_json::json!({
        "report": {
            "state": "running", "repo": "/tmp/project",
            "plan": {"goal": "Rotate tokens"}
        },
        "phase": {
            "phase": "tokens",
            "phases": [{"id": "tokens", "name": "Authentication"}]
        }
    });
    for language in ["en", "ko"] {
        for width in [792.0, 1140.0] {
            let context = egui::Context::default();
            let mut state = State::default();
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 760.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    room::show(&mut state, Some(&snapshot), false, None, language, ui);
                },
            );
            let labels = output
                .shapes
                .iter()
                .filter_map(|shape| match &shape.shape {
                    Shape::Text(text) => {
                        Some((text.galley.job.text.clone(), text.visual_bounding_rect()))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            output.textures_delta.set.clear();
            output.textures_delta.free.clear();
            let auth = if language == "ko" {
                "로컬 · CLI 인증"
            } else {
                "Local · CLI authentication"
            };
            let version = format!("v{}", env!("CARGO_PKG_VERSION"));
            assert!(
                labels.iter().all(|(label, _)| {
                    label != "AC" && label != "AgentConductor" && label != &version
                }),
                "shared branding is repeated at {width}px in {language}: {labels:?}"
            );
            for expected in ["project", auth] {
                let bounds = labels
                    .iter()
                    .find_map(|(label, bounds)| (label == expected).then_some(bounds));
                let Some(bounds) = bounds else {
                    panic!(
                        "header text {expected:?} missing at {width}px in {language}: {labels:?}"
                    );
                };
                assert!(
                    bounds.min.x >= 0.0 && bounds.max.x <= width,
                    "header text {expected:?} overflows {width}px in {language}: {bounds:?}"
                );
            }
            assert!(
                labels.iter().any(|(label, bounds)| {
                    label.contains("Authentication") && bounds.min.x >= 0.0 && bounds.max.x <= width
                }),
                "phase text overflows or is missing at {width}px in {language}: {labels:?}"
            );
            output.textures_delta.set.clear();
            output.textures_delta.free.clear();
        }
    }
}
