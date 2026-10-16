#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::appearance::{State, apply, show};
use eframe::egui::{self, accesskit::Role};

#[test]
fn appearance_menu_has_an_accessible_name() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        show(&mut state, ui);
    });
    let buttons = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(buttons.contains(&state.text("startup.settings")));
}

#[test]
fn saved_appearance_selects_matching_egui_visuals() {
    let context = egui::Context::default();
    apply("dark", &context);
    assert!(context.global_style().visuals.dark_mode);
    assert_eq!(
        context.global_style().visuals.panel_fill,
        egui::Color32::from_rgb(20, 26, 25)
    );
    assert_eq!(
        context.global_style().visuals.window_fill,
        egui::Color32::from_rgb(26, 34, 32)
    );
    assert_eq!(
        context.global_style().visuals.selection.bg_fill,
        egui::Color32::from_rgb(35, 56, 46)
    );
    assert_eq!(
        context.global_style().visuals.widgets.active.bg_fill,
        egui::Color32::from_rgb(140, 201, 168)
    );
    assert_eq!(
        context
            .global_style()
            .visuals
            .widgets
            .active
            .fg_stroke
            .color,
        egui::Color32::from_rgb(15, 26, 21)
    );
    apply("light", &context);
    assert!(!context.global_style().visuals.dark_mode);
    assert_eq!(
        context.global_style().visuals.panel_fill,
        egui::Color32::from_rgb(242, 242, 246)
    );
    assert_eq!(
        context.global_style().visuals.window_fill,
        egui::Color32::WHITE
    );
    assert_eq!(
        context.global_style().visuals.widgets.active.bg_fill,
        egui::Color32::from_rgb(10, 102, 216)
    );
    assert_eq!(
        context
            .global_style()
            .visuals
            .widgets
            .active
            .fg_stroke
            .color,
        egui::Color32::WHITE
    );
}

#[test]
fn native_tools_guide_shows_localized_labels_and_copyable_commands() {
    let commands = [
        "npx -y @agentmemory/agentmemory@latest",
        "/plugin marketplace add rohitg00/agentmemory",
        "codex plugin add agentmemory@agentmemory",
        "/plugin install ponytail@ponytail",
        "codex plugin add ponytail@ponytail",
        "brew install rtk",
        "winget install rtk-ai.rtk",
        "rtk init -g --codex",
        "uv tool install graphifyy",
        "graphify install --platform codex",
        "uv tool install --upgrade 'cocoindex-code[full]'",
        "codex mcp add cocoindex-code -- ccc mcp",
    ];
    for (language, close) in [("en", "Close"), ("ko", "닫기")] {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut open = true;
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 1000.0),
            )),
            ..Default::default()
        };
        let mut output = context.run_ui(input, |ui| {
            agentconductor::native_ui_probe::workspace::navigation::tools_guide::show(
                &mut open, language, ui,
            );
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
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        assert!(labels.contains(&close), "{close}: {labels:?}");
        for command in commands {
            let copy = if language == "en" { "Copy" } else { "복사" };
            let accessible = format!("{copy} {command}");
            assert!(
                labels.contains(&accessible.as_str()),
                "{accessible}: {labels:?}"
            );
        }
    }
}
