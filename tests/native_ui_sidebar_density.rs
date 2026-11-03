#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::sidebar;
use eframe::egui::{self, Rect, accesskit::Role};
use serde_json::json;

#[test]
fn default_sidebar_width_matches_the_main_workspace_breakpoint() {
    assert_eq!(sidebar::default_panel_width(1280.0), 277.0);
    assert_eq!(sidebar::default_panel_width(1000.0), 227.0);
}

#[test]
fn screenshot_sidebar_state_isolated_by_output_path() {
    let first = sidebar::panel_id(Some(std::ffi::OsStr::new("first.png")));
    let second = sidebar::panel_id(Some(std::ffi::OsStr::new("second.png")));
    let normal = sidebar::panel_id(None);
    assert_ne!(first, second);
    assert_eq!(normal, egui::Id::unique("native-agent-roster"));
}

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
    assert!(nodes.iter().any(|(_, node)| {
        node.role() == Role::Label && node.label() == Some(&format!("{executable} · Full access"))
    }));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn coordinator_badge_shares_the_metadata_row() {
    let profiles = vec![
        json!({"id":"claude", "parser":"claude", "permission_mode":"full_access", "catalog":{"models":["sonnet"]}}),
    ];
    let snapshot = json!({"config":{"profiles":profiles,"coordinator":"claude"}});
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
    let bounds = |label| {
        nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
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
    let coordinator = bounds("Coordinator");
    for label in ["1 models", "Full access"] {
        let badge = bounds(label);
        assert!(
            ((badge.y0 + badge.y1 - coordinator.y0 - coordinator.y1) / 2.0).abs() <= 3.0,
            "coordinator, model count and permission must share one metadata row: {coordinator:?} {badge:?}"
        );
    }
    assert!(
        coordinator.x1 <= 320.0,
        "coordinator badge escaped the rail: {coordinator:?}"
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn quota_value_and_refresh_keep_the_required_five_pixel_gap() {
    let profiles = vec![json!({"id":"codex", "parser":"codex"})];
    let snapshot = json!({
        "config":{"profiles":profiles},
        "quota_badges":{"codex":{"text":"64%/100%","title":"5h: 64% remaining"}}
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let draw = |ctx: &egui::Context| {
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(248.0, 800.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::Panel::left("quota-density")
                    .default_size(248.0)
                    .show(ctx, |ui| {
                        sidebar::show(&profiles, &snapshot, "en", true, false, ui);
                    });
            },
        )
    };
    let mut output = draw(&context);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output = draw(&context);
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
            .unwrap_or_else(|| panic!("missing {label}"))
    };
    let badge = bounds(Role::Label, "64%/100%: 5h: 64% remaining");
    let refresh = bounds(Role::Button, "Refresh codex quota");
    assert!(
        (refresh.x0 - badge.x1 - 5.0).abs() <= 1.0,
        "{badge:?} {refresh:?}"
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
