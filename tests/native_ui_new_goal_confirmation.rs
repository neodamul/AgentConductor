#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::navigation::{
    self, Event, NavigationContext, State,
};
use eframe::egui::{self, Pos2, accesskit::Role};

fn context() -> NavigationContext<'static> {
    NavigationContext {
        current_name: "Workspace",
        current_path: "/repo",
        can_change: true,
        planning_running: false,
        dirty: true,
        show_cache: false,
        language: "en",
    }
}

#[test]
fn new_goal_requires_explicit_confirmation_before_discarding_edits() {
    let ui_context = egui::Context::default();
    ui_context.enable_accesskit();
    let screen = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 700.0));
    let mut state = State::default();
    state.set_header_state(false, vec!["Workspace settings".into()]);
    state.confirm_new_goal();
    let mut emitted = None;
    let mut output = ui_context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        },
        |ui| emitted = navigation::show_in_language(&mut state, context(), ui),
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output = ui_context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        },
        |ui| emitted = navigation::show_in_language(&mut state, context(), ui),
    );
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        labels
            .iter()
            .any(|label| label.contains("Workspace settings")),
        "confirmation must name the unsaved area: {labels:?}"
    );
    assert_eq!(emitted, None, "opening the dialog must not discard edits");
    let position = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| {
            node.role() == Role::Button && node.label() == Some("Discard unsaved changes")
        })
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| {
            Pos2::new(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            )
        })
        .expect("discard confirmation button");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let mut event = None;
    let _output = ui_context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen),
            events: vec![
                egui::Event::PointerMoved(position),
                egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            ..Default::default()
        },
        |ui| event = navigation::show_in_language(&mut state, context(), ui),
    );
    assert_eq!(event, Some(Event::DiscardAndNewGoal));
}
