#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::navigation::State;
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};

#[test]
fn workspace_menu_keeps_accessible_workspace_and_help_actions() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.set_workspaces(vec![
        serde_json::json!({"name":"Current","path":"/work/current","registered":true}),
        serde_json::json!({"name":"Research","path":"/work/research","registered":true}),
    ]);
    let mut output = frame(&context, &mut state, Vec::new());
    let menu = bounds(&output, Role::Button, "Open workspace menu");
    clear(&mut output);
    let point = Pos2::new(
        ((menu.x0 + menu.x1) / 2.0) as f32,
        ((menu.y0 + menu.y1) / 2.0) as f32,
    );
    let _ = frame(
        &context,
        &mut state,
        vec![Event::PointerMoved(point), pointer(point, true)],
    );
    let mut click_frame = frame(&context, &mut state, vec![pointer(point, false)]);
    clear(&mut click_frame);
    let mut opened = frame(&context, &mut state, Vec::new());
    let labels = opened
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    clear(&mut opened);
    for label in [
        "Create workspace",
        "Open workspace Research",
        "Remove Research from list",
        "Archive workspace Research",
        "Agent tools guide…",
        "User manual",
    ] {
        assert!(
            labels.iter().any(|value| value == label),
            "{label}: {labels:?}"
        );
    }
    assert!(
        !labels
            .iter()
            .any(|label| label == "Archive workspace Current")
    );
}

fn frame(context: &egui::Context, state: &mut State, events: Vec<Event>) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(900.0, 700.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                let _ = agentconductor::native_ui_probe::workspace::navigation::show_in_language(
                    state,
                    agentconductor::native_ui_probe::workspace::navigation::NavigationContext {
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
            });
        },
    )
}

fn bounds(output: &egui::FullOutput, role: Role, label: &str) -> egui::accesskit::Rect {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap()
}

fn pointer(point: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos: point,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

fn clear(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
