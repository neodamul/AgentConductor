#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::{TaskDraft, task_controls_in_language};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::json;

fn ready_plan() -> serde_json::Value {
    json!({
        "revision":"r1",
        "proposal":{
            "phase":"ready","digest":"d1",
            "plan":{
                "workspace_mode":"shared",
                "agents":[{"id":"codex","catalog":{"models":[]}}],
                "tasks":[{
                    "id":"auth-tests","agent":"codex","model":null,
                    "prompt":"Goal:\nAdd refresh token replay tests\nDone when: old tokens fail",
                    "paths":["tests/auth.rs"],"depends_on":[],"test_command":["cargo","test"]
                },{
                    "id":"auth-docs","agent":"codex","model":null,"prompt":"Write migration notes",
                    "paths":["docs/auth.md"],"depends_on":["auth-tests"],"test_command":["cargo","test"]
                }]
            }
        }
    })
}

#[test]
fn review_task_cards_start_compact_and_expand_to_editable_fields() {
    let plan = ready_plan();
    let mut draft = TaskDraft::from_snapshot(&plan);
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut first = render(&context, &plan, &mut draft, "en", Vec::new());
    let first_labels = labels(&first);
    let summary = "1. auth-tests · Add refresh token replay tests";
    let disclosure = "Expand task: auth-tests";
    let position = first
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .find(|(_, node)| node.label() == Some(disclosure))
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| {
            Pos2::new(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            )
        });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    assert!(
        first_labels.contains(&summary.to_owned()),
        "{first_labels:?}"
    );
    assert!(
        !first_labels.contains(&"Instructions".to_owned()),
        "{first_labels:?}"
    );
    assert!(
        !first_labels.iter().any(|label| label.contains("Done when")),
        "{first_labels:?}"
    );
    assert!(
        first_labels.contains(&"Remove".to_owned()),
        "{first_labels:?}"
    );
    assert!(position.is_some(), "task disclosure control is missing");

    for pressed in [true, false] {
        let point = position.unwrap();
        let mut output = render(
            &context,
            &plan,
            &mut draft,
            "en",
            vec![
                Event::PointerMoved(point),
                Event::PointerButton {
                    pos: point,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
    }
    let mut expanded = render(&context, &plan, &mut draft, "en", Vec::new());
    let expanded_labels = labels(&expanded);
    assert!(
        expanded_labels.contains(&"Instructions".to_owned()),
        "{expanded_labels:?}"
    );
    assert!(
        expanded_labels.contains(&"Test command (JSON argv)".to_owned()),
        "{expanded_labels:?}"
    );
    expanded.textures_delta.set.clear();
    expanded.textures_delta.free.clear();
    let mut korean = render(&context, &plan, &mut draft, "ko", Vec::new());
    let korean_labels = labels(&korean);
    assert!(korean_labels.contains(&"작업 접기: auth-tests".to_owned()));
    assert!(korean_labels.contains(&"지침".to_owned()));
    korean.textures_delta.set.clear();
    korean.textures_delta.free.clear();
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

fn render(
    context: &egui::Context,
    plan: &serde_json::Value,
    draft: &mut Option<TaskDraft>,
    language: &str,
    events: Vec<Event>,
) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1000.0, 800.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            let _ = task_controls_in_language(plan, draft, false, language, ui);
        },
    )
}
