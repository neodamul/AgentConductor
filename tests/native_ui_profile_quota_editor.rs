#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::profile_tools::{Editor, show};
use eframe::egui::{self, Event as UiEvent, Modifiers, PointerButton, accesskit::Role};
use serde_json::json;

#[test]
fn existing_custom_quota_config_is_visible_in_the_profile_editor() {
    let profile = json!({
        "id":"company", "parser":"custom", "executable":"aipro", "args":[],
        "quota_command":{"args":["usage"],"timeout_seconds":15},
        "quota_rules":[{"event_type":"usage","bucket_pointer":"/account",
            "label_pointer":"/period","used_percent_pointer":"/used"}]
    });
    let mut profiles = vec![profile.clone()];
    let snapshot = json!({"config":{"profiles":profiles},"metadata_job":{},
        "metadata_queue":[],"templates":[],"quota_badges":{}});
    let mut editor = Editor::default();
    editor.open(&profile);
    let mut coordinator = String::new();
    let mut roles = agentconductor::roles::RoleRules::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900.0, 740.0));
    let mut render = |context: &egui::Context, events| {
        context.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                events,
                ..Default::default()
            },
            |ui| {
                show(
                    &mut profiles,
                    &mut coordinator,
                    &mut roles,
                    &snapshot,
                    true,
                    &mut editor,
                    ui,
                );
            },
        )
    };
    for _ in 0..2 {
        let mut output = render(&context, Vec::new());
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
    }
    let mut output = render(&context, Vec::new());
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let button = nodes
        .iter()
        .find(|(_, node)| {
            node.role() == Role::DisclosureTriangle
                && node.label() == Some("Catalog, quota and execution rules · JSON")
        })
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| {
            panic!(
                "missing quota rules disclosure; labels: {:?}",
                nodes
                    .iter()
                    .map(|(_, node)| (node.role(), node.label().map(str::to_owned)))
                    .collect::<Vec<_>>()
            )
        });
    let position = egui::pos2(
        ((button.x0 + button.x1) / 2.0) as f32,
        ((button.y0 + button.y1) / 2.0) as f32,
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let mut output = render(
        &context,
        vec![
            UiEvent::PointerMoved(position),
            UiEvent::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
            UiEvent::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    let config = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| {
            node.role() == Role::MultilineTextInput
                && node.label() == Some("Catalog, quota and execution rules · JSON")
        })
        .and_then(|(_, node)| node.value())
        .expect("expanded profile JSON editor is accessible");
    assert!(config.contains("quota_command"));
    assert!(config.contains("quota_rules"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
