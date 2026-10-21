#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::navigation::{self, NavigationContext};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};

#[test]
fn opening_workspace_menu_does_not_push_workspace_content_down() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = navigation::State::default();
    let mut baseline = frame(&context, &mut state, Vec::new());
    let content_before = bounds(&baseline, Role::Label, "workspace-content");
    let menu = bounds(&baseline, Role::Button, "Open workspace menu");
    clear(&mut baseline);

    let point = Pos2::new(
        ((menu.x0 + menu.x1) / 2.0) as f32,
        ((menu.y0 + menu.y1) / 2.0) as f32,
    );
    let _ = frame(
        &context,
        &mut state,
        vec![Event::PointerMoved(point), pointer(point, true)],
    );
    let mut click_frame = frame(&context, &mut state, vec![pointer(point, false)]);
    clear(&mut click_frame);
    let mut opened = frame(&context, &mut state, Vec::new());
    let content_after = bounds(&opened, Role::Label, "workspace-content");
    let labels = opened
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    clear(&mut opened);

    for label in ["SAVED WORKSPACES", "Create workspace", "User manual"] {
        assert!(
            labels.iter().any(|value| value == label),
            "missing {label:?}: {labels:?}"
        );
    }

    assert!(
        (content_after.y0 - content_before.y0).abs() < 1.0,
        "opening the workspace menu must overlay the header instead of shifting content: before={content_before:?}, after={content_after:?}"
    );
}

#[test]
fn opening_settings_menu_does_not_push_content_down() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = agentconductor::native_ui_probe::appearance::State::default();
    let settings_label = state.text("startup.settings").to_owned();
    let korean_language_label = state.text("startup.language_korean").to_owned();
    let mut baseline = settings_frame(&context, &mut state, Vec::new());
    let content_before = bounds(&baseline, Role::Label, "settings-content");
    let labels = baseline
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    let settings = baseline
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(settings_label.as_str()))
        .and_then(|(_, node)| node.bounds());
    clear(&mut baseline);
    let settings = settings.unwrap_or_else(|| panic!("Settings menu bounds: {labels:?}"));
    let point = Pos2::new(
        ((settings.x0 + settings.x1) / 2.0) as f32,
        ((settings.y0 + settings.y1) / 2.0) as f32,
    );
    let _ = settings_frame(
        &context,
        &mut state,
        vec![Event::PointerMoved(point), pointer(point, true)],
    );
    let mut click_frame = settings_frame(&context, &mut state, vec![pointer(point, false)]);
    clear(&mut click_frame);
    let mut opened = settings_frame(&context, &mut state, Vec::new());
    let content_after = bounds(&opened, Role::Label, "settings-content");
    let labels = opened
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    clear(&mut opened);

    assert!(
        labels.iter().any(|label| label == &korean_language_label),
        "{labels:?}"
    );
    assert!(
        (content_after.y0 - content_before.y0).abs() < 1.0,
        "opening Settings must overlay content: before={content_before:?}, after={content_after:?}"
    );
}

fn settings_frame(
    context: &egui::Context,
    state: &mut agentconductor::native_ui_probe::appearance::State,
    events: Vec<Event>,
) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(900.0, 700.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                agentconductor::native_ui_probe::appearance::show(state, ui);
                ui.label("Main content").accessible_name("settings-content");
            });
        },
    )
}

fn frame(
    context: &egui::Context,
    state: &mut navigation::State,
    events: Vec<Event>,
) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(900.0, 700.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                navigation::show_in_language(
                    state,
                    NavigationContext {
                        current_name: "Current",
                        current_path: "/work/current",
                        can_change: true,
                        planning_running: false,
                        dirty: false,
                        show_cache: false,
                        language: "en",
                    },
                    ui,
                );
                ui.label("Workspace content")
                    .accessible_name("workspace-content");
            });
        },
    )
}

fn bounds(output: &egui::FullOutput, role: Role, label: &str) -> egui::accesskit::Rect {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing {role:?} {label:?}"))
}

fn pointer(point: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos: point,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

fn clear(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
