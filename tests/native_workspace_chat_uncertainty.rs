#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::chat::{self, State};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};

#[test]
fn uncertain_chat_start_error_is_available_for_display() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.open();
    let (mut output, _) = render(&context, &mut state, Vec::new());
    let input = center(&output, Role::MultilineTextInput, None);
    clear(&mut output);
    click(&context, &mut state, input);
    let (mut output, _) = render(
        &context,
        &mut state,
        vec![Event::Paste("Continue the coordinator task".into())],
    );
    clear(&mut output);
    let (mut output, _) = render(&context, &mut state, Vec::new());
    let send = center(&output, Role::Button, Some("Send"));
    clear(&mut output);
    assert!(matches!(
        click(&context, &mut state, send),
        Some(chat::Event::Send(_))
    ));

    state.outcome_unknown("Windows denied process breakaway".into());

    assert!(state.is_uncertain());
    assert_eq!(
        state.visible_error(),
        Some("Windows denied process breakaway")
    );
}

fn render(
    context: &egui::Context,
    state: &mut State,
    events: Vec<Event>,
) -> (egui::FullOutput, Option<chat::Event>) {
    let mut result = None;
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            events,
            ..Default::default()
        },
        |ui| result = state.show("en", ui),
    );
    (output, result)
}

fn center(output: &egui::FullOutput, role: Role, label: Option<&str>) -> Pos2 {
    let bounds = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| {
            node.role() == role && label.is_none_or(|name| node.label() == Some(name))
        })
        .unwrap()
        .1
        .bounds()
        .unwrap();
    Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}

fn click(context: &egui::Context, state: &mut State, point: Pos2) -> Option<chat::Event> {
    let (mut output, _) = render(
        context,
        state,
        vec![Event::PointerMoved(point), pointer(point, true)],
    );
    clear(&mut output);
    let (mut output, event) = render(context, state, vec![pointer(point, false)]);
    clear(&mut output);
    event
}

fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

fn clear(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
