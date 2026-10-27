#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::profile_tools::{Editor, show};
use eframe::egui::{self, accesskit::Role};

#[test]
fn profile_editor_fits_and_stacks_fields_in_a_narrow_window() {
    let mut profiles = vec![serde_json::json!({
        "id":"claude", "parser":"claude", "executable":"claude", "args":[],
        "catalog":{"models":["sonnet"]}
    })];
    let snapshot = serde_json::json!({
        "config":{"profiles":[{"id":"claude"}]},
        "templates":[{"id":"claude","parser":"claude","executable":"claude","args":[]}]
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
            egui::vec2(640.0, 580.0),
        )),
        ..Default::default()
    };
    let mut render = || {
        context.run_ui(input.clone(), |ui| {
            show(
                &mut profiles,
                &mut coordinator,
                &mut roles,
                &snapshot,
                true,
                &mut editor,
                ui,
            );
        })
    };
    let mut initial = render();
    initial.textures_delta.set.clear();
    initial.textures_delta.free.clear();
    let mut output = render();
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let window = nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Window && node.label() == Some("Edit agent claude"))
        .and_then(|(_, node)| node.bounds())
        .unwrap();
    let field = |label| {
        nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .and_then(|(_, node)| node.bounds())
            .unwrap()
    };
    let provider = field("Provider");
    let agent_id = field("Agent ID");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(window.x0 >= 0.0 && window.x1 <= 640.0, "window: {window:?}");
    assert!(
        agent_id.y0 > provider.y0,
        "fields are not stacked: {provider:?}, {agent_id:?}"
    );
}
