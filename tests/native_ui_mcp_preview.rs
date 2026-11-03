#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::mcp::{SourceEvent, SourceImport};
use eframe::egui;
use serde_json::{Value, json};

#[test]
fn unsaved_goal_reference_can_start_a_preview() {
    let snapshot = json!({
        "config": {
            "goal": "Implement authentication",
            "coordinator": "claude",
            "profiles": [{"id": "claude"}],
            "mcp": {"company": {"type": "local", "command": ["company-mcp"]}}
        }
    });
    let mut goal = "Implement authentication jira:EMM-52599".to_owned();
    let mut state = SourceImport::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let (mut output, _) = frame(&context, &mut state, &mut goal, &snapshot, Vec::new());
    let button = center(&output, "Preview imports");
    clear(&mut output);
    assert_ne!(button, egui::Pos2::ZERO);

    let mut event = None;
    for pressed in [true, false] {
        let input = egui::Event::PointerButton {
            pos: button,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        let (mut output, observed) = frame(
            &context,
            &mut state,
            &mut goal,
            &snapshot,
            vec![egui::Event::PointerMoved(button), input],
        );
        clear(&mut output);
        event = observed.or(event);
    }

    assert_eq!(event, Some(SourceEvent::Preview));
}

fn frame(
    context: &egui::Context,
    state: &mut SourceImport,
    goal: &mut String,
    snapshot: &Value,
    events: Vec<egui::Event>,
) -> (egui::FullOutput, Option<SourceEvent>) {
    let mut result = None;
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1140.0, 740.0),
            )),
            events,
            ..Default::default()
        },
        |ui| result = state.show(goal, Some(snapshot), "en", true, ui),
    );
    (output, result)
}

fn center(output: &egui::FullOutput, label: &str) -> egui::Pos2 {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| {
            egui::Pos2::new(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            )
        })
        .unwrap_or(egui::Pos2::ZERO)
}

fn clear(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
