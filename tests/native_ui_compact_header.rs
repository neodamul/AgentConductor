#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::header;
use eframe::egui::{self, Shape};

#[test]
fn workspace_header_matches_the_numbered_full_width_step_row() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(860.0, 220.0),
            )),
            ..Default::default()
        },
        |ui| {
            header::show(0, "en", ui);
        },
    );
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    let numbered_badges = output
        .shapes
        .iter()
        .filter(|shape| matches!(&shape.shape, Shape::Circle(circle) if (circle.radius - 11.0).abs() < 0.1))
        .count();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    for label in [
        "1. Workspace settings",
        "2. Agent roles",
        "3. Review the plan",
    ] {
        assert!(
            labels.contains(&label.to_owned()),
            "missing numbered step label {label:?}"
        );
    }
    assert!(
        numbered_badges >= 3,
        "each step needs its numbered circle badge, found {numbered_badges}"
    );
}

#[test]
fn workspace_title_actions_match_legacy_order_and_space_between_distribution() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 180.0),
            )),
            ..Default::default()
        },
        |ui| {
            header::show_title_controls(0, "en", "startup.prepared", true, ui);
        },
    );
    let bounds = |label: &str| {
        output
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                Shape::Text(text) if text.galley.job.text == label => {
                    text.galley.rows.first().map(|row| {
                        (
                            text.pos.x + row.pos.x + row.row.visuals.mesh_bounds.left(),
                            text.pos.x + row.pos.x + row.row.visuals.mesh_bounds.right(),
                        )
                    })
                }
                _ => None,
            })
    };
    let title = bounds("What should the team work on?");
    let new_goal = bounds("New goal");
    let phase = bounds("Prepared");
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    let shape_labels = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            Shape::Text(text) => Some(text.galley.job.text.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    let title = title.unwrap_or_else(|| panic!("workspace title bounds; shapes: {shape_labels:?}"));
    let new_goal = new_goal.unwrap_or_else(|| panic!("new-goal bounds; shapes: {shape_labels:?}"));
    let phase = phase.unwrap_or_else(|| panic!("phase text bounds; labels: {labels:?}"));

    assert!(
        title.1 < new_goal.0 && new_goal.1 < phase.0,
        "legacy header order is title, New goal, then phase: title={title:?}, new_goal={new_goal:?}, phase={phase:?}"
    );
    let title_to_action = new_goal.0 - title.1;
    let action_to_phase = phase.0 - new_goal.1;
    assert!(
        (title_to_action - action_to_phase).abs() <= 16.0,
        "the legacy heading uses space-between distribution: title/action gap={title_to_action}, action/phase gap={action_to_phase}; title={title:?}, new_goal={new_goal:?}, phase={phase:?}"
    );
}
