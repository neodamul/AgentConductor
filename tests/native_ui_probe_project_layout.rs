#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::Probe;
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};

#[test]
fn project_folder_picker_stays_visible_with_coordinator_drawer_open() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = Probe::default();
    let locale = agentconductor::native_ui_probe::appearance::State::load();
    let mut first = frame(&context, &mut probe, Vec::new());
    let launcher = find_bounds(&first, Role::Button, locale.text("chat.open")).unwrap();
    clear(&mut first);
    let point = center(launcher);
    let mut pressed = frame(
        &context,
        &mut probe,
        vec![Event::PointerMoved(point), pointer(point, true)],
    );
    clear(&mut pressed);
    let mut released = frame(&context, &mut probe, vec![pointer(point, false)]);
    clear(&mut released);
    let mut opened = frame(&context, &mut probe, Vec::new());
    let browse = find_bounds(
        &opened,
        Role::Button,
        locale.text("startup.choose_your_project_folder"),
    );
    let labels = labels(&opened);
    let coordinator_edge = opened
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Splitter)
        .filter_map(|(_, node)| node.bounds())
        .map(|bounds| bounds.x0)
        .reduce(f64::max);
    clear(&mut opened);
    let browse = browse.unwrap_or_else(|| panic!("missing project picker: {labels:?}"));
    let coordinator_edge =
        coordinator_edge.unwrap_or_else(|| panic!("missing chat splitter: {labels:?}"));
    assert!(
        labels.contains(&locale.text("chat.close").to_owned()),
        "chat not open: {labels:?}"
    );

    assert!(
        browse.x1 <= coordinator_edge,
        "project picker must remain inside the workspace column: browse={browse:?}, coordinator edge={coordinator_edge}"
    );
}

fn frame(context: &egui::Context, probe: &mut Probe, events: Vec<Event>) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1140.0, 740.0),
            )),
            events,
            ..Default::default()
        },
        |ui| probe.show(ui),
    )
}

fn find_bounds(
    output: &egui::FullOutput,
    role: Role,
    label: &str,
) -> Option<egui::accesskit::Rect> {
    output
        .platform_output
        .accesskit_update
        .as_ref()?
        .nodes
        .iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
}

fn labels(output: &egui::FullOutput) -> Vec<String> {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .into_iter()
        .flat_map(|tree| tree.nodes.iter())
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect()
}

fn center(rect: egui::accesskit::Rect) -> Pos2 {
    Pos2::new(
        ((rect.x0 + rect.x1) / 2.0) as f32,
        ((rect.y0 + rect.y1) / 2.0) as f32,
    )
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
