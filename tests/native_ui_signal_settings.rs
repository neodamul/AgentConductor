#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::appearance::{self, State};
use agentconductor::native_ui_probe::workspace::signal_settings;
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};

#[test]
fn settings_menu_opens_signal_transport_preferences() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let mut action = None;
    let initial = frame(&context, &mut state, Vec::new(), &mut action);
    let settings = center(&initial, state.text("startup.settings"));
    clear(initial);

    click(&context, &mut state, settings, &mut action);
    let menu = frame(&context, &mut state, Vec::new(), &mut action);
    let labels = labels(&menu);
    assert!(
        labels
            .iter()
            .any(|label| label == state.text("startup.signal_transport")),
        "Signal transport should be reachable from Settings: {labels:?}"
    );
    let transport = center(&menu, state.text("startup.signal_transport"));
    clear(menu);
    click(&context, &mut state, transport, &mut action);
    assert_eq!(action, Some(appearance::SettingsAction::SignalTransport));
}

#[test]
fn local_rust_hides_server_port_and_retains_its_saved_value() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = signal_settings::State::default();
    state.open();
    state
        .apply_status(&serde_json::json!({
            "preference": {"transport":"agentmemory", "port":4321}
        }))
        .unwrap();
    let mut event = None;
    clear(settings_frame(&context, &mut state, Vec::new(), &mut event));
    let mut opened = settings_frame(&context, &mut state, Vec::new(), &mut event);
    let port_label = text("en", "startup.agentmemory_port");
    let visible = labels(&opened);
    clear(opened);
    assert!(
        visible.iter().any(|label| label == port_label),
        "port label missing from signal settings: {visible:?}"
    );
    opened = settings_frame(&context, &mut state, Vec::new(), &mut event);
    let input = opened
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::TextInput)
        .map(|(_, node)| node);
    let input = input.expect("agentmemory port field should be visible");
    assert_eq!(input.label(), Some(port_label));
    assert_eq!(input.value(), Some("4321"));
    let label_bounds = text_bounds(&opened, port_label);
    let input_bounds = input.bounds().unwrap();
    assert!(input_bounds.y0 >= label_bounds.max.y as f64 + 8.0);
    assert!(input_bounds.y1 - input_bounds.y0 >= 36.0);
    let local = center(&opened, text("en", "startup.local_rust"));
    clear(opened);

    click_settings(&context, &mut state, local, &mut event);
    opened = settings_frame(&context, &mut state, Vec::new(), &mut event);
    let nodes = &opened
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    assert!(!nodes.iter().any(|(_, node)| node.role() == Role::TextInput));
    assert!(!labels(&opened).iter().any(|label| label == port_label));
    clear(opened);

    let request: serde_json::Value =
        serde_json::from_slice(&state.save_request().unwrap()).unwrap();
    assert_eq!(request["preference"]["transport"], "local");
    assert_eq!(request["preference"]["port"], 4321);
}

#[test]
fn invalid_agentmemory_port_cannot_be_saved() {
    let mut state = signal_settings::State::default();
    state
        .apply_status(&serde_json::json!({
            "preference": {"transport":"agentmemory", "port":0}
        }))
        .unwrap();
    assert!(state.save_request().is_err());
}

fn frame(
    context: &egui::Context,
    state: &mut State,
    events: Vec<Event>,
    action: &mut Option<appearance::SettingsAction>,
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
        |ui| *action = appearance::show_with_cache(state, false, ui),
    )
}

fn click(
    context: &egui::Context,
    state: &mut State,
    point: Pos2,
    action: &mut Option<appearance::SettingsAction>,
) {
    for pressed in [true, false] {
        let event = Event::PointerButton {
            pos: point,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        let output = frame(
            context,
            state,
            vec![Event::PointerMoved(point), event],
            action,
        );
        clear(output);
    }
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

fn labels(output: &egui::FullOutput) -> Vec<String> {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect()
}

fn clear(mut output: egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

fn settings_frame(
    context: &egui::Context,
    state: &mut signal_settings::State,
    events: Vec<Event>,
    action: &mut Option<signal_settings::Event>,
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
        |ui| *action = state.show("en", true, false, ui),
    )
}

fn click_settings(
    context: &egui::Context,
    state: &mut signal_settings::State,
    point: Pos2,
    action: &mut Option<signal_settings::Event>,
) {
    for pressed in [true, false] {
        let event = Event::PointerButton {
            pos: point,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        let output = settings_frame(
            context,
            state,
            vec![Event::PointerMoved(point), event],
            action,
        );
        clear(output);
    }
}

fn text(language: &str, id: &'static str) -> &'static str {
    agentconductor::ui::messages::in_language(language, id)
}

fn text_bounds(output: &egui::FullOutput, text: &str) -> egui::Rect {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(shape) if shape.galley.job.text.as_str() == text => {
                Some(shape.visual_bounding_rect())
            }
            _ => None,
        })
        .find(|bounds| bounds.is_finite())
        .unwrap()
}
