#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::profile_tools::{Editor, Event, show};
use eframe::egui::{self, Event as UiEvent, Modifiers, PointerButton, Pos2, accesskit::Role};

#[test]
fn agent_profile_editor_matches_the_other_native_settings_windows() {
    let mut profiles = vec![serde_json::json!({
        "id":"claude", "parser":"claude", "executable":"claude", "args":[],
        "catalog":{"models":["sonnet"]}
    })];
    let snapshot = serde_json::json!({"config":{"profiles":[{"id":"claude"}]}});
    let mut editor = Editor::default();
    editor.open(&profiles[0].clone());
    let mut coordinator = String::new();
    let mut roles = agentconductor::roles::RoleRules::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(900.0, 740.0),
        )),
        ..Default::default()
    };
    let mut first = context.run_ui(input.clone(), |ui| {
        show(
            &mut profiles,
            &mut coordinator,
            &mut roles,
            &snapshot,
            true,
            &mut editor,
            ui,
        );
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(input, |ui| {
        show(
            &mut profiles,
            &mut coordinator,
            &mut roles,
            &snapshot,
            true,
            &mut editor,
            ui,
        );
    });
    let roles = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .map(|(_, node)| node.role())
        .collect::<Vec<_>>();
    let title = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .any(|(_, node)| node.label() == Some("Edit agent claude"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        roles.contains(&Role::Window) && !roles.contains(&Role::Dialog) && title,
        "agent profile editing should use a titled native settings window"
    );
}

#[test]
fn pending_profile_editor_disables_every_edit_field_and_save() {
    let mut profiles = vec![
        serde_json::json!({
            "id":"claude", "parser":"claude", "executable":"claude", "args":["-p"],
            "input_mode":"stdin", "max_sessions":2, "quota_refresh_minutes":5,
            "fallback":{"agent":"codex", "model":"gpt-5"},
            "catalog":{"models":["sonnet"]}
        }),
        serde_json::json!({"id":"codex", "parser":"codex", "catalog":{"models":["gpt-5"]}}),
    ];
    let snapshot = serde_json::json!({
        "config":{"profiles":[{"id":"claude"},{"id":"codex"}]}
    });
    let mut editor = Editor::default();
    editor.open(&profiles[0].clone());
    let mut coordinator = String::new();
    let mut roles = agentconductor::roles::RoleRules::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(900.0, 740.0),
        )),
        ..Default::default()
    };
    let mut first = context.run_ui(input.clone(), |ui| {
        show(
            &mut profiles,
            &mut coordinator,
            &mut roles,
            &snapshot,
            false,
            &mut editor,
            ui,
        );
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(input, |ui| {
        show(
            &mut profiles,
            &mut coordinator,
            &mut roles,
            &snapshot,
            false,
            &mut editor,
            ui,
        );
    });
    let states = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| {
            node.label()
                .map(|label| (label.to_owned(), node.is_disabled()))
        })
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for label in [
        "Agent ID",
        "Executable",
        "Arguments · JSON array",
        "Input mode",
        "Sessions",
        "Quota refresh · minutes",
        "Fallback agent",
        "Fallback model",
        "Save profile",
        "Cancel",
    ] {
        let (_, disabled) = states
            .iter()
            .find(|(name, _)| name == label)
            .unwrap_or_else(|| panic!("missing profile editor control: {label}"));
        assert!(*disabled, "{label} should be disabled while pending");
    }
}

#[test]
fn profile_editor_cancel_closes_and_save_emits_the_persistence_event() {
    let profile = serde_json::json!({
        "id":"claude", "parser":"claude", "executable":"claude", "args":[],
        "input_mode":"stdin", "max_sessions":1, "quota_refresh_minutes":5
    });
    let mut profiles = vec![profile.clone()];
    let mut editor = Editor::default();
    let context = egui::Context::default();
    context.enable_accesskit();

    editor.open(&profile);
    assert_eq!(
        click_profile_action(&context, &mut profiles, &mut editor, "Cancel"),
        None
    );
    assert!(!editor.is_editing());

    editor.open(&profile);
    editor.set_executable("/tools/claude");
    assert_eq!(
        click_profile_action(&context, &mut profiles, &mut editor, "Save profile"),
        Some(Event::SaveProfiles)
    );
    assert!(
        editor.is_editing(),
        "wait for the save response before closing the editor"
    );
    assert_eq!(
        profiles,
        vec![profile],
        "Save must stage edits until persistence succeeds"
    );
}

fn click_profile_action(
    context: &egui::Context,
    profiles: &mut Vec<serde_json::Value>,
    editor: &mut Editor,
    target: &str,
) -> Option<Event> {
    let (_, mut output) = render_profile(context, profiles, editor, Vec::new());
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let (_, mut output) = render_profile(context, profiles, editor, Vec::new());
    let position = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(target))
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| {
            Pos2::new(
                (bounds.x0 + bounds.x1) as f32 / 2.0,
                (bounds.y0 + bounds.y1) as f32 / 2.0,
            )
        })
        .unwrap_or_else(|| panic!("missing {target:?} button"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for pressed in [true, false] {
        let button = UiEvent::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        let (event, mut output) = render_profile(
            context,
            profiles,
            editor,
            vec![UiEvent::PointerMoved(position), button],
        );
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        if !pressed {
            return event;
        }
    }
    unreachable!()
}

fn render_profile(
    context: &egui::Context,
    profiles: &mut Vec<serde_json::Value>,
    editor: &mut Editor,
    events: Vec<UiEvent>,
) -> (Option<Event>, egui::FullOutput) {
    let snapshot = serde_json::json!({
        "config":{"profiles":profiles},
        "templates":[], "metadata_job":{}, "metadata_queue":[], "quota_badges":{}
    });
    let mut coordinator = "claude".to_owned();
    let mut roles = agentconductor::roles::RoleRules::default();
    let mut event = None;
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(900.0, 740.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            event = show(
                profiles,
                &mut coordinator,
                &mut roles,
                &snapshot,
                true,
                editor,
                ui,
            );
        },
    );
    (event, output)
}
