#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::profile_tools::{
    Editor, Event, RenderContext, catalog_request, parse_manual_catalog, show_in_language,
};
use eframe::egui::{self, Event as InputEvent, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::json;

#[test]
fn manual_model_ids_are_deduplicated_and_keep_operator_provenance() {
    let catalog = parse_manual_catalog("large\nsmall\nlarge\n", 42, "en").unwrap();
    assert_eq!(catalog.models, ["large", "small"]);
    assert_eq!(
        catalog.source,
        "Operator-configured model IDs; CLI discovery unavailable"
    );
    assert_eq!(catalog.queried_at, 42);
    assert_eq!(catalog.cli_version, "Not exposed by CLI");
}

#[test]
fn observed_json_catalog_is_retained_and_invalid_entries_are_rejected() {
    let input =
        r#"{"models":["company/large"],"source":"observed","queried_at":7,"cli_version":"v1"}"#;
    let catalog = parse_manual_catalog(input, 42, "unused").unwrap();
    assert_eq!(catalog.models, ["company/large"]);
    assert_eq!(catalog.source, "observed");
    assert_eq!(catalog.queried_at, 7);
    assert_eq!(catalog.cli_version, "v1");
    for invalid in ["", "bad model", "--unsafe", "\0model"] {
        assert!(parse_manual_catalog(invalid, 42, "en").is_err());
    }
}

#[test]
fn oversized_observed_json_is_rejected_before_parsing() {
    let oversized = format!(
        r#"{{"models":["company/large"],"source":"{}","queried_at":7,"cli_version":"v1"}}"#,
        "x".repeat(256 * 1024)
    );
    assert!(parse_manual_catalog(&oversized, 42, "en").is_err());
}

#[test]
fn custom_agent_models_action_opens_catalog_editor_and_save_is_revision_bound() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut profiles = vec![json!({"id":"aipro","parser":"custom","catalog":null})];
    let snapshot =
        json!({"revision":"r7","config":{"profiles":[{"id":"aipro","parser":"custom"}]}});
    let mut editor = Editor::default();
    let mut coordinator = "aipro".to_owned();
    let mut roles = agentconductor::roles::RoleRules::default();
    editor.open_catalog(&profiles[0]);
    let (mut output, _) = frame(
        &context,
        &mut profiles,
        &snapshot,
        &mut coordinator,
        &mut roles,
        &mut editor,
        vec![],
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let (mut output, _) = frame(
        &context,
        &mut profiles,
        &snapshot,
        &mut coordinator,
        &mut roles,
        &mut editor,
        vec![],
    );
    let current = labels(&output);
    assert!(
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::MultilineTextInput),
        "labels={current:?}"
    );
    assert!(current.iter().any(|label| label == "Save model IDs"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    editor.set_catalog_text("company-small\ncompany-large");
    let (output, _) = frame(
        &context,
        &mut profiles,
        &snapshot,
        &mut coordinator,
        &mut roles,
        &mut editor,
        vec![],
    );
    let (output, action) = click(
        &context,
        &mut profiles,
        &snapshot,
        &mut coordinator,
        &mut roles,
        &mut editor,
        button(&output, "Save model IDs"),
    );
    assert!(matches!(action, Some(Event::SetCatalog { agent, catalog })
        if agent == "aipro" && catalog.models == ["company-small", "company-large"]));
    drop(output);
}

#[test]
fn catalog_save_uses_the_existing_revision_bound_startup_dispatch() {
    let catalog = parse_manual_catalog("company-small", 42, "en").unwrap();
    let packet = catalog_request("r9", "aipro", &catalog);
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["kind"], "catalog");
    assert_eq!(value["revision"], "r9");
    assert_eq!(value["agent"], "aipro");
    assert_eq!(value["catalog"]["models"][0], "company-small");
}

fn frame(
    context: &egui::Context,
    profiles: &mut Vec<serde_json::Value>,
    snapshot: &serde_json::Value,
    coordinator: &mut String,
    roles: &mut agentconductor::roles::RoleRules,
    editor: &mut Editor,
    events: Vec<InputEvent>,
) -> (egui::FullOutput, Option<Event>) {
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
            action = show_in_language(
                profiles,
                coordinator,
                roles,
                editor,
                RenderContext {
                    snapshot,
                    editable: true,
                    language: "en",
                    show_registry: true,
                },
                ui,
            )
        },
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    (output, action)
}

fn click(
    context: &egui::Context,
    profiles: &mut Vec<serde_json::Value>,
    snapshot: &serde_json::Value,
    coordinator: &mut String,
    roles: &mut agentconductor::roles::RoleRules,
    editor: &mut Editor,
    position: Pos2,
) -> (egui::FullOutput, Option<Event>) {
    let mut result = None;
    for pressed in [true, false] {
        let event = InputEvent::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        result = Some(frame(
            context,
            profiles,
            snapshot,
            coordinator,
            roles,
            editor,
            vec![InputEvent::PointerMoved(position), event],
        ));
    }
    result.unwrap()
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

fn button(output: &egui::FullOutput, label: &str) -> Pos2 {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .find_map(|(_, node)| {
            (node.label() == Some(label))
                .then(|| node.bounds())
                .flatten()
        })
        .map(|bounds| {
            Pos2::new(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            )
        })
        .unwrap_or_else(|| panic!("button not found: {label}; {:?}", labels(output)))
}
