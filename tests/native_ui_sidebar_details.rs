#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::sidebar;
use eframe::egui::{self, accesskit::Role};
use serde_json::json;

#[test]
fn agent_card_exposes_executable_and_permission_without_hover() {
    let profile = json!({
        "id":"claude",
        "parser":"claude",
        "executable":"/tmp/ac-demo/bin/claude",
        "permission_mode":"operator_auto_approve",
        "max_sessions":3
    });
    let profiles = vec![profile];
    let snapshot = json!({"config":{"coordinator":"claude","profiles":profiles}});
    for (language, expected) in [
        ("en", "/tmp/ac-demo/bin/claude · approves its own tools"),
        ("ko", "/tmp/ac-demo/bin/claude · 자체 도구 자동 승인"),
    ] {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = context.run_ui(Default::default(), |ui| {
            sidebar::show(&profiles, &snapshot, language, true, false, ui);
        });
        let nodes = &output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes;
        let visible = nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Label && node.label() == Some(expected));
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        assert!(visible, "missing visible card details for {language}");
    }
}

#[test]
fn agent_card_badges_match_the_reference_metadata_row() {
    use eframe::egui::Rect;
    let profiles = vec![json!({
        "id":"claude", "parser":"claude", "executable":"/tmp/ac-demo/bin/claude",
        "permission_mode":"operator_auto_approve", "max_sessions":3,
        "catalog":{"models":["sonnet","opus","haiku"]}
    })];
    let snapshot = json!({"config":{"coordinator":"claude","profiles":profiles}});
    let context = egui::Context::default();
    context.enable_accesskit();
    let draw = || {
        context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(320.0, 800.0),
                )),
                ..Default::default()
            },
            |ui| {
                sidebar::show(&profiles, &snapshot, "en", true, false, ui);
            },
        )
    };
    let mut output = draw();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output = draw();
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let bounds = |label: &str| {
        nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .and_then(|(_, node)| node.bounds())
            .unwrap_or_else(|| panic!("missing visible metadata badge {label:?}"))
    };
    let coordinator = bounds("Coordinator");
    let models = bounds("3 models");
    let permission = bounds("approves its own tools");
    for badge in [models, permission] {
        assert!(
            ((badge.y0 + badge.y1 - coordinator.y0 - coordinator.y1) / 2.0).abs() <= 3.0,
            "coordinator, model count and permission must share a metadata row: {coordinator:?} {badge:?}"
        );
    }
    output.textures_delta.clear();
}
