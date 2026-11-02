#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::chat::State;
use agentconductor::native_ui_probe::workspace::chat::coordinator_chat_issue;
use eframe::egui::{self, Pos2, accesskit::Role};

#[test]
fn incompatible_coordinator_shows_reason_and_disables_send() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.open();
    state.set_compatibility_issue(Some(
        "Coordinator aipro requires read-only capabilities".into(),
    ));
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        },
        |ui| {
            state.show("en", ui);
        },
    );
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let reason_visible = nodes.iter().any(|(_, node)| {
        node.label().is_some_and(|label| {
            label.contains("Coordinator aipro requires read-only capabilities")
        })
    });
    let send_disabled = nodes.iter().any(|(_, node)| {
        node.role() == Role::Button && node.label() == Some("Send") && node.is_disabled()
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(reason_visible);
    assert!(send_disabled);
}

#[test]
fn compatibility_reason_uses_the_selected_coordinator_profile() {
    let snapshot = serde_json::json!({
        "config": {
            "coordinator":"aipro",
            "profiles":[{
                "id":"aipro", "parser":"custom", "executable":"aipro",
                "args":["run"], "permission_mode":"unverified"
            }]
        }
    });
    let issue = coordinator_chat_issue(&snapshot).unwrap();
    assert!(issue.contains("read-only capabilities"));
}
