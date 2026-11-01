#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::appearance::{SettingsAction, State, show_with_cache};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2};

#[test]
fn settings_opens_the_agent_guide_and_user_manual() {
    assert_eq!(
        click_setting("startup.agent_tools_guide"),
        Some(SettingsAction::ToolsGuide)
    );
    assert_eq!(
        click_setting("startup.user_manual"),
        Some(SettingsAction::UserManual)
    );
}

fn click_setting(label: &'static str) -> Option<SettingsAction> {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let (_, mut output) = frame(&context, &mut state, Vec::new());
    let settings = center(&output, state.text("startup.settings"));
    clear(&mut output);
    click(&context, &mut state, settings);
    let (_, mut menu) = frame(&context, &mut state, Vec::new());
    let menu_label = state.text(label);
    let position = center(&menu, menu_label);
    clear(&mut menu);
    click(&context, &mut state, position)
}

fn click(context: &egui::Context, state: &mut State, position: Pos2) -> Option<SettingsAction> {
    for pressed in [true, false] {
        let (action, mut output) = frame(
            context,
            state,
            vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        clear(&mut output);
        if !pressed {
            return action;
        }
    }
    unreachable!()
}

fn clear(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

fn frame(
    context: &egui::Context,
    state: &mut State,
    events: Vec<Event>,
) -> (Option<SettingsAction>, egui::FullOutput) {
    let mut action = None;
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(900.0, 600.0),
            )),
            events,
            ..Default::default()
        },
        |ui| action = show_with_cache(state, true, ui),
    );
    (action, output)
}

fn center(output: &egui::FullOutput, label: &str) -> Pos2 {
    let bounds = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.label() == Some(label))
        .unwrap()
        .bounds()
        .unwrap();
    Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}
