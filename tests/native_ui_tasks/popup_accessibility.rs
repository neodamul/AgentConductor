use agentconductor::native_ui_probe::workspace::{TaskDraft, task_controls_in_language};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, accesskit::Role};

#[test]
fn task_editor_exposes_assignment_instructions_scope_dependencies_and_command() {
    let mut value = super::snapshot();
    value["proposal"]["plan"]["agents"] = serde_json::json!([{
        "id":"codex", "executable":"codex", "args":[]
    }]);
    value["proposal"]["plan"]["tasks"] = serde_json::json!([super::task()]);
    value["proposal"]["plan"]["workspace_mode"] = serde_json::json!("shared");
    let mut draft = TaskDraft::from_snapshot(&value);
    let context = context();
    open_first_editor(&context, &value, &mut draft, "en");
    let labels = labels(&context, &value, &mut draft, "en");
    for label in [
        "Assigned agent",
        "Session model",
        "Instructions",
        "Owned paths (one per line)",
        "Dependencies (comma separated)",
        "Test command (JSON argv)",
    ] {
        assert!(
            labels.iter().any(|actual| actual == label),
            "{label}: {labels:?}"
        );
    }
}

#[test]
fn task_editor_uses_korean_labels_and_accessible_names() {
    let mut value = super::snapshot();
    value["proposal"]["plan"]["agents"] = serde_json::json!([{
        "id":"codex", "executable":"codex", "args":[]
    }]);
    value["proposal"]["plan"]["tasks"] = serde_json::json!([super::task(), {
        "id":"auth-docs", "agent":"codex", "model":null, "prompt":"Write migration note",
        "paths":["docs/auth.md"], "depends_on":["auth-tests"], "test_command":["cargo","test"]
    }]);
    value["proposal"]["plan"]["workspace_mode"] = serde_json::json!("shared");
    let mut draft = TaskDraft::from_snapshot(&value);
    draft.as_mut().unwrap().tasks[0]["prompt"] = "Updated instructions".into();
    let context = context();
    open_first_editor(&context, &value, &mut draft, "ko");
    let labels = labels(&context, &value, &mut draft, "ko");
    for label in [
        "작업 ID",
        "배정된 에이전트",
        "세션 모델",
        "지침",
        "담당 경로 (한 줄에 하나)",
        "의존성 (쉼표로 구분)",
        "테스트 명령 (JSON argv)",
        "작업 추가",
        "계획 변경 검토 필요",
        "변경된 계획 저장 후 다시 검토",
        "작업 삭제",
    ] {
        assert!(
            labels.iter().any(|actual| actual == label),
            "{label}: {labels:?}"
        );
    }
}

#[test]
fn opening_task_editor_does_not_push_the_next_task_down() {
    let mut value = super::snapshot();
    value["proposal"]["plan"]["agents"] = serde_json::json!([{
        "id":"codex", "executable":"codex", "args":[]
    }]);
    value["proposal"]["plan"]["tasks"] = serde_json::json!([super::task(), {
        "id":"auth-docs", "agent":"codex", "model":null, "prompt":"Write migration note",
        "paths":["docs/auth.md"], "depends_on":["auth-tests"], "test_command":["cargo","test"]
    }]);
    value["proposal"]["plan"]["workspace_mode"] = "shared".into();
    let mut draft = TaskDraft::from_snapshot(&value);
    let context = context();
    let before = render(&context, &value, &mut draft, "en", Vec::new());
    let before_y = label_y(&before, "02 · auth-docs");
    clear(before);
    open_first_editor(&context, &value, &mut draft, "en");
    let opened = render(&context, &value, &mut draft, "en", Vec::new());
    let after_y = label_y(&opened, "02 · auth-docs");
    clear(opened);
    assert_eq!(after_y, before_y, "task details must overlay the task list");
}

fn context() -> egui::Context {
    let context = egui::Context::default();
    context.enable_accesskit();
    context
}

fn open_first_editor(
    context: &egui::Context,
    value: &serde_json::Value,
    draft: &mut Option<TaskDraft>,
    language: &str,
) {
    let label = if language == "ko" {
        "작업 펼치기: auth-tests"
    } else {
        "Expand task: auth-tests"
    };
    let first = render(
        context,
        value,
        draft,
        language,
        vec![Event::PointerMoved(Pos2::new(850.0, 700.0))],
    );
    let rect = first
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap();
    clear(first);
    let point = Pos2::new(
        ((rect.x0 + rect.x1) / 2.0) as f32,
        ((rect.y0 + rect.y1) / 2.0) as f32,
    );
    for pressed in [true, false] {
        clear(render(
            context,
            value,
            draft,
            language,
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
    clear(render(context, value, draft, language, Vec::new()));
}

fn labels(
    context: &egui::Context,
    value: &serde_json::Value,
    draft: &mut Option<TaskDraft>,
    language: &str,
) -> Vec<String> {
    let output = render(context, value, draft, language, Vec::new());
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect();
    clear(output);
    labels
}

fn label_y(output: &egui::FullOutput, label: &str) -> f32 {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| bounds.y0 as f32)
        .unwrap_or_else(|| panic!("missing accessible UI node {label:?}"))
}

fn render(
    context: &egui::Context,
    value: &serde_json::Value,
    draft: &mut Option<TaskDraft>,
    language: &str,
    events: Vec<Event>,
) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(900.0, 740.0))),
            events,
            ..Default::default()
        },
        |ui| {
            let _ = task_controls_in_language(value, draft, false, false, language, ui);
        },
    )
}

fn clear(mut output: egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
