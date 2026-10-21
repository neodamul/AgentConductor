#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::{TaskDraft, task_controls_in_language};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};

#[test]
fn review_plan_task_editor_uses_bounded_available_width() {
    let snapshot = serde_json::json!({
        "proposal": {
            "phase": "ready",
            "digest": "digest-1",
            "plan": {
                "agents": [{"id":"codex", "catalog":{"models":[]}}],
                "workspace_mode": "shared",
                "tasks": [{
                    "id":"tests", "agent":"codex", "model":null,
                    "prompt":"Implement and test the requested behavior",
                    "paths":["tests/feature.rs"], "depends_on":[], "test_command":["cargo","test"]
                }]
            }
        }
    });
    let mut draft = TaskDraft::from_snapshot(&snapshot);
    let context = egui::Context::default();
    context.enable_accesskit();
    let first = frame(
        &context,
        &snapshot,
        &mut draft,
        vec![Event::PointerMoved(Pos2::new(850.0, 700.0))],
    );
    let labels = first
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    let rect = first
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Expand task: tests"))
        .and_then(|(_, node)| node.bounds());
    clear(first);
    let rect = rect.unwrap_or_else(|| panic!("task disclosure missing from {labels:?}"));
    let point = Pos2::new(
        ((rect.x0 + rect.x1) / 2.0) as f32,
        ((rect.y0 + rect.y1) / 2.0) as f32,
    );
    for pressed in [true, false] {
        clear(frame(
            &context,
            &snapshot,
            &mut draft,
            vec![
                Event::PointerMoved(point),
                Event::PointerButton {
                    pos: point,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        ));
    }
    let mut output = frame(&context, &snapshot, &mut draft, Vec::new());
    let inputs = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::TextInput)
        .filter_map(|(_, node)| {
            node.bounds()
                .map(|bounds| (node.label().map(str::to_owned), bounds))
        })
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let bounds = inputs
        .iter()
        .find(|(label, _)| label.as_deref() == Some("Owned paths (one per line)"))
        .map(|(_, bounds)| *bounds)
        .unwrap_or_else(|| panic!("instructions input missing from {inputs:?}"));
    let width = bounds.x1 - bounds.x0;
    assert!(
        (520.0..=560.0).contains(&width),
        "the Review Plan path editor should fill its bounded popup, got {bounds:?}"
    );
}

fn frame(
    context: &egui::Context,
    snapshot: &serde_json::Value,
    draft: &mut Option<TaskDraft>,
    events: Vec<Event>,
) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(900.0, 740.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            let _ = task_controls_in_language(snapshot, draft, false, false, "en", ui);
        },
    )
}

fn clear(mut output: egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
