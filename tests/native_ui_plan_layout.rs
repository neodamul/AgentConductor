#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::{TaskDraft, task_controls_in_language};
use eframe::egui::{self, accesskit::Role};

#[test]
fn review_plan_task_editor_uses_available_width() {
    let snapshot = serde_json::json!({
        "proposal": {
            "phase": "ready",
            "digest": "digest-1",
            "plan": {
                "agents": [{"id":"codex", "catalog":{"models":[]}}],
                "workspace_mode": "shared",
                "tasks": [{
                    "id":"tests", "agent":"codex", "model":null,
                    "prompt":"Implement and test the requested behavior",
                    "paths":["tests/feature.rs"], "depends_on":[], "test_command":["cargo","test"]
                }]
            }
        }
    });
    let mut draft = TaskDraft::from_snapshot(&snapshot);
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900.0, 740.0),
            )),
            ..Default::default()
        },
        |ui| {
            let _ = task_controls_in_language(&snapshot, &mut draft, false, "en", ui);
        },
    );
    let inputs = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::TextInput)
        .filter_map(|(_, node)| {
            node.bounds()
                .map(|bounds| (node.label().map(str::to_owned), bounds))
        })
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let bounds = inputs
        .iter()
        .find(|(label, _)| label.as_deref() == Some("Owned paths (one per line)"))
        .map(|(_, bounds)| *bounds)
        .unwrap_or_else(|| panic!("instructions input missing from {inputs:?}"));
    assert!(
        bounds.x1 - bounds.x0 >= 600.0,
        "the Review Plan path editor should use the available content width, got {bounds:?}"
    );
}
