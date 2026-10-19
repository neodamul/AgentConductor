#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::goals;
use eframe::egui::{self, Pos2, Rect};
use serde_json::{Value, json};

#[test]
fn goal_list_shows_each_slug_and_localized_stage() {
    let context = egui::Context::default();
    let (mut output, _) = render(&context, &snapshot());
    let visible = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for expected in [
        "refresh_tokens",
        "in progress",
        "billing_api",
        "analysis",
        "Improve billing API",
    ] {
        assert!(
            visible.contains(expected),
            "missing {expected:?} in {visible:?}"
        );
    }
}

#[test]
fn reviewed_phase_is_labeled_as_under_review() {
    let mut snapshot = snapshot();
    snapshot["proposal"]["slug_phase"] = json!("02-docs");
    let context = egui::Context::default();
    let (mut output, _) = render(&context, &snapshot);
    let visible = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        visible.contains("Reviewing phase 1/1: Documentation"),
        "reviewed phase heading is missing: {visible}"
    );
}

fn snapshot() -> Value {
    json!({
        "job":{"state":"idle"},
        "proposal":{"slug_phase":null},
        "slugs":[
            {"slug":"refresh_tokens","goal":"Rotate refresh tokens","active":true,
             "progress":{"stage":"phase","current_phase":"02-docs","phases":[
                 {"id":"02-docs","name":"Documentation","stage":"task","task":[],"verification":[],"completed":[]}
             ],"mismatches":[]},"plan":{"phases":[]}},
            {"slug":"billing_api","goal":"Improve billing API","active":false,
             "progress":{"stage":"analysis","phases":[],"mismatches":[]}}
        ]
    })
}

fn render(context: &egui::Context, snapshot: &Value) -> (egui::FullOutput, Option<goals::Event>) {
    let mut event = None;
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(900.0, 700.0))),
            ..Default::default()
        },
        |ui| event = goals::show(snapshot, "en", false, ui),
    );
    (output, event)
}
