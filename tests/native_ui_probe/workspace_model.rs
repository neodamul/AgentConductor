use super::super::Probe;
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2};
use serde_json::{Value, json};

#[test]
fn workspace_coordinator_model_is_selectable_from_observed_catalog() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = probe(custom_profile());
    let first = frame(&context, &mut probe, Vec::new());
    let selector = position(&first, &localized("startup.coordinator_model"));

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
    let selector = position(&first, &localized("startup.coordinator_model"));

    click(&context, &mut probe, selector);
    let open = frame(&context, &mut probe, Vec::new());
    let default_option = position(&open, &localized("startup.cli_default"));
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
    let language = crate::native_ui_probe::appearance::State::load()
        .language()
        .to_owned();
    let reason = crate::native_ui_probe::workspace::chat::coordinator_issue(
        &probe.profiles,
        &probe.coordinator,
    )
    .unwrap();
    let expected = crate::ui::messages::interpolate(
        crate::ui::messages::in_language(&language, "startup.coordinator_cannot_chat"),
        &[("value1", &reason)],
    );
    assert!(
        labels.contains(&expected.as_str()),
        "missing {expected}; labels: {labels:?}"
    );
}

#[test]
fn workspace_coordinator_fields_fit_the_default_native_viewport() {
    use eframe::egui::Rect;
    let locale = crate::native_ui_probe::appearance::State::load();
    let model_label = locale.text("startup.coordinator_model");
    let isolation_label = locale.text("startup.isolate_tasks_in_git_worktrees");
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = probe(custom_profile());
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1140.0, 740.0))),
            ..Default::default()
        },
        |ui| probe.update_ui(ui),
    );
    let (name_width, coordinator_width, model_width, model_bounds, isolation_bounds) = {
        let nodes = &output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes;
        let field_width = |label: &str| {
            nodes
                .iter()
                .filter(|(_, node)| node.label() == Some(label))
                .filter_map(|(_, node)| node.bounds())
                .map(|bounds| bounds.x1 - bounds.x0)
                .max_by(f64::total_cmp)
                .unwrap_or_else(|| panic!("missing field {label:?}"))
        };
        let field_bounds = |label: &str| {
            nodes
                .iter()
                .find(|(_, node)| node.label() == Some(label))
                .and_then(|(_, node)| node.bounds())
        };
        (
            field_width(locale.text("startup.workspace_name")),
            field_width(locale.text("startup.planning_coordinator")),
            field_width(model_label),
            field_bounds(model_label),
            field_bounds(isolation_label),
        )
    };
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        name_width >= coordinator_width * 1.5,
        "workspace name should receive the wider legacy column: name={name_width}, coordinator={coordinator_width}"
    );
    assert!(
        (coordinator_width - model_width).abs() <= 10.0,
        "coordinator and model columns should be balanced: coordinator={coordinator_width}, model={model_width}"
    );
    for (label, bounds) in [
        (model_label, model_bounds),
        (isolation_label, isolation_bounds),
    ] {
        let bounds = bounds.unwrap_or_else(|| panic!("missing {label}"));
        assert!(
            bounds.x1 <= 1132.0,
            "{label} overflows the viewport: {bounds:?}"
        );
    }
}

#[test]
fn workspace_isolation_checkbox_selects_git_worktrees() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = probe(custom_profile());
    let first = frame(&context, &mut probe, Vec::new());
    let checkbox = position(&first, &localized("startup.isolate_tasks_in_git_worktrees"));

    click(&context, &mut probe, checkbox);

    assert_eq!(probe.workspace_mode, crate::plan::WorkspaceMode::Worktree);
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
    assert_eq!(saved["config"]["workspace_mode"], "worktree");
}

#[test]
fn coordinator_blocker_uses_a_visible_warning_panel() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = probe(custom_profile());
    let output = frame(&context, &mut probe, Vec::new());

    let panel_width = output
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::epaint::Shape::Rect(rect) if rect.fill.a() == 28 => Some(rect.rect.width()),
            _ => None,
        })
        .max_by(f32::total_cmp)
        .unwrap_or_default();
    assert!(
        panel_width >= 500.0,
        "warning panel width was {panel_width}"
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

fn localized(key: &'static str) -> String {
    let locale = crate::native_ui_probe::appearance::State::load();
    locale.text(key).to_owned()
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
