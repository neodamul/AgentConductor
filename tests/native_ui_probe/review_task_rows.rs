use super::{super::show, Probe};
use eframe::egui::{self, Rect, Vec2};
use serde_json::json;

#[test]
fn full_review_screen_exposes_all_three_manual_plan_task_rows() {
    let tasks = (1..=3)
        .map(|index| {
            json!({
                "id":format!("manual-{index}"),
                "agent":"codex",
                "role":"development",
                "model":null,
                "prompt":format!("Implement scoped part {index}"),
                "paths":[format!("src/part-{index}.rs")],
                "depends_on":[],
                "test_command":["cargo", "test"]
            })
        })
        .collect::<Vec<_>>();
    let snapshot = json!({
        "revision":"r1",
        "config":{
            "name":"Review fixture",
            "repository":"/private/tmp/review-fixture",
            "goal":"Create three independent scoped changes",
            "coordinator":"codex",
            "workspace_mode":"shared",
            "profiles":[{"id":"codex","catalog":{"models":[]}}],
            "roles":{}
        },
        "proposal":{
            "phase":"ready",
            "digest":"digest-1",
            "plan":{
                "goal":"Create three independent scoped changes",
                "workspace_mode":"shared",
                "agents":[{"id":"codex","catalog":{"models":[]}}],
                "tasks":tasks
            }
        }
    });
    let mut probe = Probe {
        workspace_step: 2,
        snapshot: Some(snapshot),
        ..Default::default()
    };
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(1140.0, 772.0),
            )),
            ..Default::default()
        },
        |ui| show(&mut probe, ui),
    );
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for id in ["manual-1", "manual-2", "manual-3"] {
        let label = crate::ui::messages::interpolate(
            crate::ui::messages::in_language(
                probe.navigation.appearance.language(),
                "startup.expand_task",
            ),
            &[("value1", id)],
        );
        let bounds = nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label.as_str()))
            .and_then(|(_, node)| node.bounds());
        assert!(
            bounds.is_some_and(|bounds| {
                bounds.x0 >= 0.0 && bounds.y0 >= 0.0 && bounds.x1 <= 1140.0 && bounds.y1 <= 772.0
            }),
            "missing or clipped {id} in full Review screen: {bounds:?}"
        );
    }
}
