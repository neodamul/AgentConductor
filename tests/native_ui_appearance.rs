#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::appearance::{
    SettingsAction, State, apply, show, show_with_cache,
};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};

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
fn cache_cleanup_is_reached_through_settings() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let mut action = None;
    let mut output = frame(&context, &mut state, Vec::new(), &mut action);
    let settings = center(&output, state.text("startup.settings"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    let (_, mut menu) = click(&context, &mut state, settings);
    menu.textures_delta.set.clear();
    menu.textures_delta.free.clear();
    let mut menu = frame(&context, &mut state, Vec::new(), &mut action);
    let cache = center(&menu, state.text("startup.cache_cleanup_accessible"));
    menu.textures_delta.set.clear();
    menu.textures_delta.free.clear();

    let (action, mut result) = click(&context, &mut state, cache);
    result.textures_delta.set.clear();
    result.textures_delta.free.clear();
    assert_eq!(action, Some(SettingsAction::Cache));
}

#[test]
fn workspace_settings_sharing_is_reached_through_settings() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let mut output = frame(&context, &mut state, Vec::new(), &mut None);
    let settings = center(&output, state.text("startup.settings"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    let (_, mut menu) = click(&context, &mut state, settings);
    menu.textures_delta.set.clear();
    menu.textures_delta.free.clear();
    let menu = frame(&context, &mut state, Vec::new(), &mut None);
    let sharing = center(&menu, state.text("startup.workspace_settings_sharing"));
    let (action, mut result) = click(&context, &mut state, sharing);
    result.textures_delta.set.clear();
    result.textures_delta.free.clear();

    assert_eq!(action, Some(SettingsAction::WorkspaceSharing));
}

fn click(
    context: &egui::Context,
    state: &mut State,
    position: Pos2,
) -> (Option<SettingsAction>, egui::FullOutput) {
    let mut action = None;
    for pressed in [true, false] {
        let event = Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        let mut output = frame(
            context,
            state,
            vec![Event::PointerMoved(position), event],
            &mut action,
        );
        if pressed {
            output.textures_delta.set.clear();
            output.textures_delta.free.clear();
        } else {
            return (action, output);
        }
    }
    unreachable!()
}

fn frame(
    context: &egui::Context,
    state: &mut State,
    events: Vec<Event>,
    action: &mut Option<SettingsAction>,
) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(900.0, 600.0),
            )),
            events,
            ..Default::default()
        },
        |ui| *action = show_with_cache(state, true, ui),
    )
}

fn center(output: &egui::FullOutput, label: &str) -> Pos2 {
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let bounds = nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.label() == Some(label))
        .unwrap_or_else(|| {
            let labels = nodes
                .iter()
                .filter_map(|(_, node)| node.label())
                .collect::<Vec<_>>();
            panic!("missing {label:?}: {labels:?}")
        })
        .bounds()
        .unwrap();
    Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
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
        egui::Color32::from_rgb(58, 72, 68)
    );
    assert_eq!(
        context
            .global_style()
            .visuals
            .widgets
            .active
            .fg_stroke
            .color,
        egui::Color32::from_rgb(221, 230, 225)
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
        egui::Color32::from_rgb(196, 196, 204)
    );
    assert_eq!(
        context
            .global_style()
            .visuals
            .widgets
            .active
            .fg_stroke
            .color,
        egui::Color32::from_rgb(28, 28, 30)
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
        let mut output = None;
        for _ in 0..2 {
            let next = context.run_ui(input.clone(), |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    agentconductor::native_ui_probe::workspace::navigation::tools_guide::show(
                        &mut open, language, ui,
                    );
                });
            });
            if let Some(mut previous) = output.replace(next) {
                previous.textures_delta.set.clear();
                previous.textures_delta.free.clear();
            }
        }
        let mut output = output.unwrap();
        let labels = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        assert!(
            labels.iter().any(|label| label == close),
            "{close}: {labels:?}"
        );
        for command in commands.iter().take(6) {
            assert!(
                labels.iter().any(|label| label == command),
                "command missing from tools guide: {command}"
            );
        }
        let copy = if language == "en" { "Copy" } else { "복사" };
        assert!(
            labels.iter().filter(|label| label.as_str() == copy).count() >= 6,
            "visible commands need copy buttons"
        );
    }
}
