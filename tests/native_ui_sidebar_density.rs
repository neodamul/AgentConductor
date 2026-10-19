#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::sidebar;
use eframe::egui::{self, Rect, accesskit::Role};
use serde_json::json;

#[test]
fn sidebar_keeps_agent_command_accessible_on_the_name_row() {
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
    output = context.run_ui(Default::default(), |ctx| {
        egui::Panel::left("sidebar-density")
            .default_size(248.0)
            .show(ctx, |ui| {
                sidebar::show(&profiles, &snapshot, "en", true, false, ui);
            });
    });

    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let accessible_name = format!("claude · {details}");
    let node = nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Label && node.label() == Some(&accessible_name))
        .and_then(|(_, node)| node.bounds())
        .expect("full executable and permission remain accessible on the agent name");
    assert!(
        node.height() <= 16.0,
        "agent name should stay one line: {node:?}"
    );
    assert!(!nodes.iter().any(|(_, node)| node.label() == Some(&details)));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn coordinator_badge_shares_the_agent_action_row() {
    let profiles = vec![json!({"id":"claude", "parser":"claude"})];
    let snapshot = json!({"config":{"profiles":profiles,"coordinator":"claude"}});
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
            egui::Panel::left("coordinator-density")
                .default_size(248.0)
                .show(ctx, |ui| {
                    sidebar::show(&profiles, &snapshot, "en", true, false, ui);
                });
        },
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output = context.run_ui(Default::default(), |ctx| {
        egui::Panel::left("coordinator-density")
            .default_size(248.0)
            .show(ctx, |ui| {
                sidebar::show(&profiles, &snapshot, "en", true, false, ui);
            });
    });

    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let bounds = |role, label| {
        nodes
            .iter()
            .find(|(_, node)| node.role() == role && node.label() == Some(label))
            .and_then(|(_, node)| node.bounds())
            .unwrap_or_else(|| {
                panic!(
                    "missing {label}; available labels: {:?}",
                    nodes
                        .iter()
                        .filter_map(|(_, node)| node.label())
                        .collect::<Vec<_>>()
                )
            })
    };
    let actions = bounds(Role::Button, "Edit agent claude");
    let coordinator = bounds(Role::Label, "Coordinator");
    assert!(
        ((actions.y0 + actions.y1) / 2.0 - (coordinator.y0 + coordinator.y1) / 2.0).abs() <= 3.0,
        "coordinator badge is on a separate card row: {actions:?} {coordinator:?}"
    );
    assert!(
        coordinator.x1 <= 248.0,
        "coordinator badge escaped the rail: {coordinator:?}"
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
