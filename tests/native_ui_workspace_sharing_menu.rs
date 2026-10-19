#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::Probe;
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2};

#[test]
fn sharing_actions_live_in_settings_and_open_their_dialog() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = Probe::default();
    let mut output = frame(&context, &mut probe, Vec::new());
    let initial_labels = labels(&output);
    let language = ["en", "ko"]
        .into_iter()
        .find(|language| {
            initial_labels.iter().any(|label| {
                label == agentconductor::ui::messages::in_language(*language, "startup.settings")
            })
        })
        .expect("settings button has a supported language");
    let message = |id| agentconductor::ui::messages::in_language(language, id);
    assert!(
        !initial_labels
            .iter()
            .any(|label| label == message("startup.export_settings_button"))
    );
    assert!(
        !initial_labels
            .iter()
            .any(|label| label == message("startup.import_settings_button"))
    );
    let settings = center(&output, message("startup.settings"));
    clear(&mut output);
    let settings = settings.expect("settings button is accessible");

    let mut output = click(&context, &mut probe, settings);
    clear(&mut output);
    let mut output = frame(&context, &mut probe, Vec::new());
    let sharing = center(&output, message("startup.workspace_settings_sharing"));
    let menu_labels = labels(&output);
    clear(&mut output);
    let sharing = sharing
        .unwrap_or_else(|| panic!("workspace sharing is reachable from settings: {menu_labels:?}"));

    let mut output = click(&context, &mut probe, sharing);
    clear(&mut output);
    let mut output = frame(&context, &mut probe, Vec::new());
    let labels = labels(&output);
    clear(&mut output);
    assert!(
        labels
            .iter()
            .any(|label| label == message("startup.export_workspace_settings_accessible")),
        "sharing dialog is open: {labels:?}"
    );
    assert!(
        labels
            .iter()
            .any(|label| label == message("startup.import_workspace_settings_accessible"))
    );
    assert_eq!(
        labels
            .iter()
            .filter(|label| label.as_str() == message("startup.workspace_settings_sharing"))
            .count(),
        1,
        "sharing dialog should have one heading: {labels:?}"
    );
}

fn click(context: &egui::Context, probe: &mut Probe, point: Pos2) -> egui::FullOutput {
    let mut output = frame(
        context,
        probe,
        vec![Event::PointerMoved(point), pointer(point, true)],
    );
    clear(&mut output);
    frame(context, probe, vec![pointer(point, false)])
}

fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
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

fn center(output: &egui::FullOutput, target: &str) -> Option<Pos2> {
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let bounds = nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.label() == Some(target))
        .and_then(|node| node.bounds())?;
    Some(Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    ))
}

fn clear(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
