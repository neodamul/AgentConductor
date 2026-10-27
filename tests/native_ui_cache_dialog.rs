#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::Probe;
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};

#[test]
fn cache_dialog_close_button_closes_the_window() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = Probe::default();
    let mut output = frame(&context, &mut probe, Vec::new());
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let mut output = frame(&context, &mut probe, Vec::new());
    let settings = center(&output, |label| matches!(label, "설정" | "Settings"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let mut output = click(&context, &mut probe, settings);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output = frame(&context, &mut probe, Vec::new());
    let cache = center(&output, |label| {
        label.contains("캐시") || label.contains("cache")
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let mut output = click(&context, &mut probe, cache);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output = frame(&context, &mut probe, Vec::new());
    let close = center(&output, |label| matches!(label, "닫기" | "Close"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    let mut output = click(&context, &mut probe, close);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output = frame(&context, &mut probe, Vec::new());
    let labels = labels(&output);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        !labels
            .iter()
            .any(|label| label == "닫기" || label == "Close"),
        "cache dialog remained open after close: {labels:?}"
    );
}

fn click(context: &egui::Context, probe: &mut Probe, position: Pos2) -> egui::FullOutput {
    let mut output = None;
    for pressed in [true, false] {
        let event = Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        let next = frame(context, probe, vec![Event::PointerMoved(position), event]);
        if let Some(mut previous) = output.replace(next) {
            previous.textures_delta.set.clear();
            previous.textures_delta.free.clear();
        }
    }
    output.unwrap()
}

fn frame(context: &egui::Context, probe: &mut Probe, events: Vec<Event>) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(900.0, 600.0),
            )),
            events,
            ..Default::default()
        },
        |ui| probe.show(ui),
    )
}

fn center(output: &egui::FullOutput, matches: impl Fn(&str) -> bool) -> Pos2 {
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let (label, bounds) = nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .find_map(|(_, node)| {
            let label = node.label()?;
            matches(label).then_some((label, node.bounds()?))
        })
        .unwrap_or_else(|| panic!("target button not found: {:?}", labels(output)));
    let _ = label;
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
        .filter(|(_, node)| node.role() == Role::Button)
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect()
}
