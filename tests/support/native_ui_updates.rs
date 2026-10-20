use agentconductor::native_ui_probe::updates::{Action, State};
use eframe::egui::{self, Event, PointerButton, Pos2, accesskit::Role};

pub fn render_update(
    context: &egui::Context,
    state: &mut State,
    language: &str,
    pending: bool,
    events: Vec<Event>,
) -> egui::FullOutput {
    context.run_ui(update_input(events), |ui| {
        state.show(language, pending, ui);
    })
}

pub fn click_update(
    context: &egui::Context,
    state: &mut State,
    language: &str,
    pending: bool,
    position: Pos2,
) -> Option<Action> {
    let mut action = None;
    for pressed in [true, false] {
        let event = Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        let mut output = context.run_ui(
            update_input(vec![Event::PointerMoved(position), event]),
            |ui| action = state.show(language, pending, ui),
        );
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
    }
    action
}

fn update_input(events: Vec<Event>) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(800.0, 600.0),
        )),
        events,
        ..Default::default()
    }
}

pub fn update_button_center(output: &mut egui::FullOutput, label: &str) -> Pos2 {
    let center = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| {
            Pos2::new(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            )
        })
        .unwrap_or_else(|| panic!("button {label:?} should have screen bounds"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    center
}
