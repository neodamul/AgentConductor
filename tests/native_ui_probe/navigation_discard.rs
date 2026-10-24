use super::Probe;
use crate::native_ui_probe::workspace::navigation::{
    Event, NavigationContext, State, show_in_language,
};
use eframe::egui::{self, Event as InputEvent, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::json;

fn input(events: Vec<InputEvent>) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(1280.0, 800.0),
        )),
        events,
        ..Default::default()
    }
}

fn frame(probe: &mut Probe, context: &egui::Context, events: Vec<InputEvent>) {
    let mut output = context.run_ui(input(events), |ui| probe.show_navigation(ui));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

fn click_probe(probe: &mut Probe, context: &egui::Context, label: &str) {
    frame(probe, context, vec![]);
    let mut output = context.run_ui(input(vec![]), |ui| probe.show_navigation(ui));
    let update = output.platform_output.accesskit_update.take().unwrap();
    let labels: Vec<_> = update
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect();
    let bounds = update
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing button {label}; labels: {labels:?}"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let pos = Pos2::new(
        (bounds.x0 + bounds.x1) as f32 / 2.0,
        (bounds.y0 + bounds.y1) as f32 / 2.0,
    );
    for pressed in [true, false] {
        frame(
            probe,
            context,
            vec![
                InputEvent::PointerMoved(pos),
                InputEvent::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
}

#[test]
fn discard_restores_the_saved_workspace_and_drops_only_local_drafts() {
    let mode = serde_json::to_value(crate::plan::WorkspaceMode::default()).unwrap();
    let roles = serde_json::to_value(crate::roles::RoleRules::default()).unwrap();
    let snapshot = json!({
        "config": {
            "name": "Saved", "repository": "/saved", "goal": "Saved goal",
            "workspace_mode": mode, "coordinator": "saved-agent",
            "profiles": [{"id":"saved-agent"}], "roles": roles
        },
        "proposal": {
            "digest": "saved-digest",
            "plan": {"tasks": [{"id":"saved-task"}], "workspace_mode": mode}
        }
    });
    let mut draft = crate::native_ui_probe::workspace::TaskDraft::from_snapshot(&snapshot).unwrap();
    draft.tasks = vec![json!({"id":"draft-task"})];
    let mut probe = Probe {
        snapshot: Some(snapshot),
        name: "Draft".into(),
        repository: "/draft".into(),
        goal: "Draft goal".into(),
        profiles: vec![json!({"id":"draft-agent"})],
        task_draft: Some(draft),
        navigation: State {
            name: "New workspace draft".into(),
            ..Default::default()
        },
        workspace_step: 2,
        ..Default::default()
    };

    assert_eq!(
        probe.dirty_areas(probe.snapshot.as_ref(), "ko"),
        ["워크스페이스 설정", "에이전트 프로필", "계획 작업"]
    );

    probe.discard_workspace_changes();

    assert_eq!(probe.name, "Saved");
    assert_eq!(probe.repository, "/saved");
    assert_eq!(probe.goal, "Saved goal");
    assert_eq!(probe.profiles, vec![json!({"id":"saved-agent"})]);
    assert!(probe.task_draft.is_none());
    assert!(probe.navigation.name.is_empty());
    assert_eq!(probe.workspace_step, 0);
    assert!(probe.worker.is_none());
    assert!(probe.pending.is_none());
}

#[test]
fn keep_editing_cancels_and_confirmed_discard_restores_saved_fields() {
    let mode = serde_json::to_value(crate::plan::WorkspaceMode::default()).unwrap();
    let roles = serde_json::to_value(crate::roles::RoleRules::default()).unwrap();
    let mut probe = Probe {
        snapshot: Some(json!({"config":{
            "name":"Saved","repository":"/saved","goal":"Saved goal",
            "workspace_mode":mode,"coordinator":"saved-agent",
            "profiles":[{"id":"saved-agent"}],"roles":roles
        }})),
        name: "Draft".into(),
        repository: "/draft".into(),
        goal: "Draft goal".into(),
        ..Default::default()
    };
    let context = egui::Context::default();
    context.enable_accesskit();

    let language = probe.navigation.appearance.language().to_owned();
    click_probe(
        &mut probe,
        &context,
        crate::ui::messages::in_language(&language, "startup.discard_changes"),
    );
    click_probe(
        &mut probe,
        &context,
        crate::ui::messages::in_language(&language, "startup.keep_editing"),
    );
    assert_eq!(probe.name, "Draft");
    assert_eq!(probe.repository, "/draft");
    assert_eq!(probe.goal, "Draft goal");

    click_probe(
        &mut probe,
        &context,
        crate::ui::messages::in_language(&language, "startup.discard_changes"),
    );
    click_probe(
        &mut probe,
        &context,
        crate::ui::messages::in_language(&language, "startup.discard_unsaved_changes"),
    );
    assert_eq!(probe.name, "Saved");
    assert_eq!(probe.repository, "/saved");
    assert_eq!(probe.goal, "Saved goal");
    assert!(probe.worker.is_none());
    assert!(probe.pending.is_none());
}

#[test]
fn clicking_back_to_run_emits_the_open_run_event() {
    let mut state = State::default();
    state.set_header_state(true, Vec::new());
    let context = egui::Context::default();
    context.enable_accesskit();
    let navigation = NavigationContext {
        current_name: "Saved",
        current_path: "/saved",
        can_change: true,
        planning_running: false,
        dirty: false,
        show_cache: true,
        language: "en",
    };
    let click = |state: &mut State| {
        let mut output = context.run_ui(input(vec![]), |ui| {
            show_in_language(state, navigation, ui);
        });
        let update = output.platform_output.accesskit_update.take().unwrap();
        let bounds = update
            .nodes
            .into_iter()
            .find(|(_, node)| node.role() == Role::Button && node.label() == Some("← Back to run"))
            .and_then(|(_, node)| node.bounds())
            .unwrap();
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        Pos2::new(
            (bounds.x0 + bounds.x1) as f32 / 2.0,
            (bounds.y0 + bounds.y1) as f32 / 2.0,
        )
    };
    click(&mut state);
    let pos = click(&mut state);
    let mut event = None;
    for pressed in [true, false] {
        let mut output = context.run_ui(
            input(vec![
                InputEvent::PointerMoved(pos),
                InputEvent::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ]),
            |ui| event = show_in_language(&mut state, navigation, ui),
        );
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
    }
    assert_eq!(event, Some(Event::BackToRun));
}
