#![cfg(feature = "native-ui-probe")]

use eframe::egui::{self, accesskit::Role};

#[test]
fn profile_edit_and_remove_use_vector_icons_with_accessible_names() {
    let mut profiles = vec![serde_json::json!({
        "id":"claude", "parser":"claude", "catalog":{"models":["sonnet"]}
    })];
    let snapshot = serde_json::json!({"config":{"profiles":[{"id":"claude"}]}});
    let mut coordinator = String::new();
    let mut roles = agentconductor::roles::RoleRules::default();
    let mut editor = agentconductor::native_ui_probe::workspace::profile_tools::Editor::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 400.0),
            )),
            ..Default::default()
        },
        |ui| {
            agentconductor::native_ui_probe::workspace::profile_tools::show_in_language(
                &mut profiles,
                &mut coordinator,
                &mut roles,
                &mut editor,
                agentconductor::native_ui_probe::workspace::profile_tools::RenderContext {
                    snapshot: &snapshot,
                    editable: true,
                    language: "en",
                    show_registry: true,
                },
                ui,
            );
        },
    );
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    for label in ["Edit agent claude", "Remove agent claude"] {
        let bounds = nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
            .and_then(|(_, node)| node.bounds())
            .expect("profile action remains accessible");
        assert!(output.shapes.iter().any(|shape| match &shape.shape {
            egui::Shape::LineSegment { points, .. } => points.iter().all(|point| {
                f64::from(point.x) >= bounds.x0
                    && f64::from(point.x) <= bounds.x1
                    && f64::from(point.y) >= bounds.y0
                    && f64::from(point.y) <= bounds.y1
            }),
            _ => false,
        }));
    }
    assert!(output.shapes.iter().any(|shape| matches!(
        &shape.shape,
        egui::Shape::Text(text) if text.galley.job.text == "Models ↻"
    )));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
