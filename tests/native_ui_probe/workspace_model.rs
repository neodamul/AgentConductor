use super::super::Probe;
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2};
use serde_json::{Value, json};

#[test]
fn workspace_coordinator_model_is_selectable_from_observed_catalog() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = probe(custom_profile());
    let first = frame(&context, &mut probe, Vec::new());
    let selector = position(&first, "조정자 모델");

    click(&context, &mut probe, selector);
    let open = frame(&context, &mut probe, Vec::new());
    let option = position(&open, "opus");
    click(&context, &mut probe, option);

    assert_eq!(probe.profiles[0]["coordinator_model"], "opus");
    let request = crate::native_ui_probe::workspace::workspace_save_request(
        probe.snapshot.as_ref().unwrap(),
        &probe.name,
        &probe.repository,
        &probe.goal,
        probe.workspace_mode,
        &probe.profiles,
        &probe.coordinator,
    );
    let saved: Value = serde_json::from_slice(&request).unwrap();
    assert_eq!(saved["config"]["profiles"][0]["coordinator_model"], "opus");
}

#[test]
fn workspace_cli_default_removes_the_coordinator_model_override() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = probe(custom_profile());
    let first = frame(&context, &mut probe, Vec::new());
    let selector = position(&first, "조정자 모델");

    click(&context, &mut probe, selector);
    let open = frame(&context, &mut probe, Vec::new());
    let default_option = position(&open, "CLI 기본값");
    click(&context, &mut probe, default_option);

    assert!(probe.profiles[0].get("coordinator_model").is_none());
}

#[test]
fn workspace_shows_read_only_coordinator_blocker_before_chat_opens() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = probe(custom_profile());
    assert!(
        crate::native_ui_probe::workspace::chat::coordinator_issue(
            &probe.profiles,
            &probe.coordinator
        )
        .is_some()
    );
    let output = frame(&context, &mut probe, Vec::new());

    let labels = labels(&output);
    assert!(
        labels
            .iter()
            .any(|label| label.starts_with("이 코디네이터와는 대화할 수 없습니다:")),
        "labels: {labels:?}"
    );
}

fn probe(profile: Value) -> Probe {
    let coordinator = profile["id"].as_str().unwrap().to_owned();
    Probe {
        profiles: vec![profile.clone()],
        coordinator: coordinator.clone(),
        snapshot: Some(json!({
            "config": {
                "profiles": [profile],
                "coordinator": coordinator,
                "goal": "Review a service health endpoint"
            },
            "job": {"kind": "idle", "state": "completed"}
        })),
        name: "Health endpoint".into(),
        repository: "/tmp".into(),
        goal: "Review a service health endpoint".into(),
        ..Probe::default()
    }
}

fn custom_profile() -> Value {
    json!({
        "id": "aipro",
        "parser": "custom",
        "executable": "aipro",
        "args": ["run", "{task_file}"],
        "permission_mode": "operator_auto_approve",
        "catalog": {
            "models": ["sonnet", "opus"],
            "source": "aipro models",
            "queried_at": 1,
            "cli_version": "test"
        },
        "coordinator_model": "sonnet"
    })
}

fn frame(context: &egui::Context, probe: &mut Probe, events: Vec<Event>) -> egui::FullOutput {
    let input = egui::RawInput {
        events,
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| super::show(probe, ui));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output
}

fn click(context: &egui::Context, probe: &mut Probe, point: Pos2) {
    for pressed in [true, false] {
        frame(
            context,
            probe,
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
    }
}

fn position(output: &egui::FullOutput, label: &str) -> Pos2 {
    let update = output.platform_output.accesskit_update.as_ref().unwrap();
    let bounds = update
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing {label}; labels: {:?}", labels(output)));
    Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}

fn labels(output: &egui::FullOutput) -> Vec<&str> {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect()
}
