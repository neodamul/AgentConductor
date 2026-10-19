#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::sidebar;
use eframe::egui::{self, Rect, accesskit::Role};
use serde_json::json;

#[test]
fn sidebar_keeps_agent_command_accessible_without_wrapping_the_card() {
    let executable = "/opt/company/agents/installation/claude-code";
    let details = format!("{executable} · full_access");
    let profiles = vec![json!({
        "id":"claude", "parser":"claude", "executable":executable,
        "permission_mode":"full_access", "max_sessions":3
    })];
    let snapshot = json!({"config":{"profiles":profiles}});
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(248.0, 800.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::Panel::left("sidebar-density")
                .default_size(248.0)
                .show(ctx, |ui| {
                    sidebar::show(&profiles, &snapshot, "en", true, false, ui);
                });
        },
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output = context.run_ui(Default::default(), |ui| {
        sidebar::show(&profiles, &snapshot, "en", true, false, ui);
    });

    let node = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Label && node.label() == Some(&details))
        .map(|(_, node)| node)
        .expect("full executable and permission remain accessible");
    assert!(
        node.bounds().unwrap().height() <= 16.0,
        "agent command wraps into extra card rows: {:?}",
        node.bounds()
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
