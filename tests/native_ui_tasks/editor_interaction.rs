use agentconductor::native_ui_probe::workspace::{TaskDraft, task_controls_in_language};
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, Rect};

#[test]
fn agent_combo_selection_does_not_dismiss_the_task_editor() {
    let mut value = super::snapshot();
    value["proposal"]["plan"]["agents"] = serde_json::json!([
        {"id":"codex", "executable":"codex", "args":[]},
        {"id":"fixture-agent", "executable":"fixture-agent", "args":[]}
    ]);
    value["proposal"]["plan"]["tasks"] = serde_json::json!([super::task()]);
    let mut draft = TaskDraft::from_snapshot(&value);
    let context = egui::Context::default();
    context.enable_accesskit();
    open_first_editor(&context, &value, &mut draft);
    click_label(&context, &value, &mut draft, "Assigned agent");

    let menu = render(&context, &value, &mut draft, Vec::new());
    let nodes = &menu
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let option = nodes
        .iter()
        .find(|(_, node)| node.label() == Some("fixture-agent"))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("agent option must remain visible"));
    let point = Pos2::new(
        ((option.x0 + option.x1) / 2.0) as f32,
        ((option.y0 + option.y1) / 2.0) as f32,
    );
    clear(menu);
    click_point(&context, &value, &mut draft, point);
    let output = render(&context, &value, &mut draft, Vec::new());
    let labels: Vec<_> = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect();
    clear(output);
    assert!(
        labels
            .iter()
            .any(|label| label == "Collapse task: auth-tests")
    );
    assert!(labels.iter().any(|label| label == "Assigned agent"));
    assert_eq!(draft.as_ref().unwrap().tasks[0]["agent"], "fixture-agent");
}

#[test]
fn task_editor_closes_on_escape_and_outside_click() {
    let value = super::snapshot();
    let mut value = value;
    value["proposal"]["plan"]["tasks"] = serde_json::json!([super::task()]);
    let mut draft = TaskDraft::from_snapshot(&value);
    let context = egui::Context::default();
    context.enable_accesskit();
    open_first_editor(&context, &value, &mut draft);

    clear(render(
        &context,
        &value,
        &mut draft,
        vec![Event::Key {
            key: Key::Escape,
            physical_key: Some(Key::Escape),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    ));
    assert_editor_closed(&context, &value, &mut draft);

    open_first_editor(&context, &value, &mut draft);
    click_point(&context, &value, &mut draft, Pos2::new(850.0, 700.0));
    assert_editor_closed(&context, &value, &mut draft);
}

fn click_label(
    context: &egui::Context,
    value: &serde_json::Value,
    draft: &mut Option<TaskDraft>,
    label: &str,
) {
    let output = render(context, value, draft, Vec::new());
    let rect = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing UI control {label}"));
    let point = Pos2::new(
        ((rect.x0 + rect.x1) / 2.0) as f32,
        ((rect.y0 + rect.y1) / 2.0) as f32,
    );
    clear(output);
    click_point(context, value, draft, point);
}

fn click_point(
    context: &egui::Context,
    value: &serde_json::Value,
    draft: &mut Option<TaskDraft>,
    point: Pos2,
) {
    for pressed in [true, false] {
        clear(render(
            context,
            value,
            draft,
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
}

fn assert_editor_closed(
    context: &egui::Context,
    value: &serde_json::Value,
    draft: &mut Option<TaskDraft>,
) {
    let output = render(context, value, draft, Vec::new());
    let labels: Vec<_> = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect();
    clear(output);
    assert!(
        labels
            .iter()
            .any(|label| label == "Expand task: auth-tests")
    );
}

fn open_first_editor(
    context: &egui::Context,
    value: &serde_json::Value,
    draft: &mut Option<TaskDraft>,
) {
    let output = render(context, value, draft, Vec::new());
    let rect = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Expand task: auth-tests"))
        .and_then(|(_, node)| node.bounds())
        .unwrap();
    let point = Pos2::new(
        ((rect.x0 + rect.x1) / 2.0) as f32,
        ((rect.y0 + rect.y1) / 2.0) as f32,
    );
    clear(output);
    click_point(context, value, draft, point);
    clear(render(context, value, draft, Vec::new()));
}

fn render(
    context: &egui::Context,
    value: &serde_json::Value,
    draft: &mut Option<TaskDraft>,
    events: Vec<Event>,
) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(900.0, 740.0))),
            events,
            ..Default::default()
        },
        |ui| {
            let _ = task_controls_in_language(value, draft, false, false, "en", ui);
        },
    )
}

fn clear(mut output: egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
