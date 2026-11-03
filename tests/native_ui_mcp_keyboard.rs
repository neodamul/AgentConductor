#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::mcp::{Event as McpEvent, State};
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::json;

#[test]
fn tab_leaves_the_mcp_editor_and_does_not_insert_whitespace() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut editor = State::default();
    let snapshot = json!({"config": {}});
    editor.open(Some(&snapshot));
    frame(&context, &mut editor, &snapshot, Vec::new());
    let (output, _) = frame(&context, &mut editor, &snapshot, Vec::new());
    let (before, position) = editor_value(&output);
    frame(
        &context,
        &mut editor,
        &snapshot,
        vec![
            Event::PointerMoved(position),
            Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    frame(
        &context,
        &mut editor,
        &snapshot,
        vec![Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );

    let (output, _) = frame(
        &context,
        &mut editor,
        &snapshot,
        vec![Event::Key {
            key: Key::Tab,
            physical_key: Some(Key::Tab),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );

    assert_eq!(editor_value(&output).0, before);
    assert!(
        output
            .platform_output
            .accesskit_update
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Button
                && node.label() == Some("Save MCP settings"))
    );
    let (_, action) = frame(
        &context,
        &mut editor,
        &snapshot,
        vec![Event::Key {
            key: Key::Enter,
            physical_key: Some(Key::Enter),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    assert!(matches!(action, Some(McpEvent::Save(settings)) if settings == before));
}

fn frame(
    context: &egui::Context,
    editor: &mut State,
    snapshot: &serde_json::Value,
    events: Vec<Event>,
) -> (egui::FullOutput, Option<McpEvent>) {
    let mut action = None;
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1000.0, 800.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            action = editor.show(Some(snapshot), "en", true, ui);
        },
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    (output, action)
}

fn editor_value(output: &egui::FullOutput) -> (String, Pos2) {
    let node = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.role() == Role::MultilineTextInput)
        .unwrap();
    let bounds = node.bounds().unwrap();
    (
        node.value().unwrap().to_owned(),
        Pos2::new(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        ),
    )
}
