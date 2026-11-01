use super::{Probe, frame, labels};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};
use std::{
    thread,
    time::{Duration, Instant},
};

pub(super) fn wait_for_ui(
    context: &egui::Context,
    probe: &mut Probe,
    ready: impl Fn(&[String]) -> bool,
) {
    wait_for_ui_timeout(context, probe, Duration::from_secs(15), ready);
}

pub(super) fn wait_for_ui_timeout(
    context: &egui::Context,
    probe: &mut Probe,
    timeout: Duration,
    ready: impl Fn(&[String]) -> bool,
) {
    let deadline = Instant::now() + timeout;
    loop {
        let mut output = frame(context, probe, Vec::new());
        let labels = labels(&output);
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        if ready(&labels) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "UI state not reached; labels: {labels:?}"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

pub(super) fn wait_for_enabled_button(context: &egui::Context, probe: &mut Probe, label: &str) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let mut output = frame(context, probe, Vec::new());
        let buttons = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter_map(|(_, node)| {
                (node.role() == Role::Button)
                    .then(|| node.label().map(|name| (name, node.is_disabled())))
                    .flatten()
            })
            .collect::<Vec<_>>();
        let enabled = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| {
                node.role() == Role::Button && node.label() == Some(label) && !node.is_disabled()
            });
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        if enabled {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "enabled button {label:?} missing; buttons (disabled): {buttons:?}; labels: {:?}",
            labels(&output)
        );
        thread::sleep(Duration::from_millis(20));
    }
}

pub(super) fn click(context: &egui::Context, probe: &mut Probe, label: &str) -> egui::FullOutput {
    let mut output = frame(context, probe, Vec::new());
    let mut position = button_position(&output, label);
    for _ in 0..6 {
        if position.y <= 1760.0 {
            break;
        }
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        output = frame(
            context,
            probe,
            vec![
                Event::PointerMoved(Pos2::new(600.0, 900.0)),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -600.0),
                    phase: egui::TouchPhase::Move,
                    source: egui::MouseWheelSource::Wheel,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        position = button_position(&output, label);
    }
    assert!(
        position.y <= 1760.0,
        "button {label:?} remains offscreen at {position:?}"
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    frame(
        context,
        probe,
        vec![
            Event::PointerMoved(position),
            Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
            Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
    )
}

fn button_position(output: &egui::FullOutput, label: &str) -> Pos2 {
    let bounds = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("button {label:?} missing; labels: {:?}", labels(output)));
    Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}
