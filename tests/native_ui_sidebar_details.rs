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
