#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::navigation::{
    Event as NavigationEvent, NavigationContext, State, show_in_language,
};
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, accesskit::Role};

#[test]
fn pressing_enter_submits_a_new_workspace_name() {
    let mut state = State::default();
    state.open();
    let context = egui::Context::default();
    context.enable_accesskit();
    let show = |state: &mut State, events: Vec<Event>| {
        let mut action = None;
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    egui::vec2(1000.0, 800.0),
                )),
                events,
                ..Default::default()
            },
            |ui| {
                action = show_in_language(
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
            },
        );
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        (output, action)
    };

    let (output, _) = show(&mut state, Vec::new());
    let bounds = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.role() == Role::TextInput)
        .unwrap()
        .bounds()
        .unwrap();
    let position = Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    show(
        &mut state,
        vec![
            Event::PointerMoved(position),
            Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    show(
        &mut state,
        vec![Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    show(&mut state, vec![Event::Text("Research".into())]);
    let (_, action) = show(
        &mut state,
        vec![Event::Key {
            key: Key::Enter,
            physical_key: Some(Key::Enter),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );

    assert_eq!(action, Some(NavigationEvent::Create("Research".into())));
}
