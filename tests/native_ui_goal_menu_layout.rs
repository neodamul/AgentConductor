#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::goals;
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};

#[test]
fn opening_phase_details_does_not_push_goal_content_down() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let phases = (0..8)
        .map(|index| {
            serde_json::json!({
                "id": format!("phase-{index}"),
                "name": format!("Phase {index}"),
                "stage": "planned"
            })
        })
        .collect::<Vec<_>>();
    let planned = (0..8)
        .map(|index| {
            serde_json::json!({
                "id": format!("phase-{index}"),
                "brief": if index == 7 { "Details" } else { "Earlier phase" },
                "specs": if index == 7 {
                    (0..20)
                        .map(|task| serde_json::json!({"id": format!("task-{task}"), "title": format!("Long task {task}")}))
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                }
            })
        })
        .collect::<Vec<_>>();
    let snapshot = serde_json::json!({
        "slugs": [{
            "slug": "goal",
            "goal": "Goal",
            "active": true,
            "progress": {
                "phases": phases,
                "current_phase": "phase-7"
            },
            "plan": {"analysis": "", "phases": planned}
        }],
        "job": {"state": "idle"}
    });
    let mut baseline = frame(&context, &snapshot, Vec::new());
    let before = bounds(&baseline, Role::Label, "goal-content");
    let menu = baseline
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| {
            node.role() == Role::Button && node.label().is_some_and(|label| label.starts_with("8."))
        })
        .and_then(|(_, node)| node.bounds());
    clear(&mut baseline);
    let menu = menu.unwrap_or_else(|| panic!("missing phase menu"));

    let point = Pos2::new(
        ((menu.x0 + menu.x1) / 2.0) as f32,
        ((menu.y0 + menu.y1) / 2.0) as f32,
    );
    let _ = frame(
        &context,
        &snapshot,
        vec![Event::PointerMoved(point), pointer(point, true)],
    );
    let mut click_frame = frame(&context, &snapshot, vec![pointer(point, false)]);
    clear(&mut click_frame);
    let mut opened = frame(&context, &snapshot, Vec::new());
    let after = bounds(&opened, Role::Label, "goal-content");
    let labels = opened
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    clear(&mut opened);

    assert!(labels.iter().any(|label| label == "Details"), "{labels:?}");
    assert!(
        (after.y0 - before.y0).abs() < 1.0,
        "opening phase details must overlay the goal card: before={before:?}, after={after:?}"
    );
}

fn frame(
    context: &egui::Context,
    snapshot: &serde_json::Value,
    events: Vec<Event>,
) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(900.0, 480.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.label("Goal content").accessible_name("goal-content");
                        goals::show(snapshot, "en", false, ui);
                    });
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
        .unwrap_or_else(|| panic!("missing {role:?} {label:?}"))
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
