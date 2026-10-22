#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::{
    coordinator_feedback,
    task_controls::{self, Action},
};
use eframe::egui;
use serde_json::{Value, json};

#[test]
fn coordinator_request_is_revision_bound_and_limited() {
    let snapshot =
        json!({"revision":"r1","report":{"state":"failed","plan":{"tasks":[]},"tasks":[]}});
    let packet = task_controls::request(
        &snapshot,
        Action::CoordinatorRequest {
            message: "Explain the verification failure".into(),
        },
    )
    .unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&packet).unwrap(),
        json!({"kind":"action","revision":"r1","action":{"kind":"coordinator_request","message":"Explain the verification failure"}})
    );
    for message in [
        String::new(),
        "  ".into(),
        "x".repeat(8193),
        "bad\0text".into(),
    ] {
        assert!(task_controls::request(&snapshot, Action::CoordinatorRequest { message }).is_err());
    }
}

#[test]
fn coordinator_panel_names_recipient_and_request_states() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let judgments = json!({"coordinator":"claude","decision_count":1,"judgments":[{"kind":"request","judgment":"processing","detail":"Please investigate"}]});
    let mut draft = String::new();
    let mut output = context.run_ui(Default::default(), |ui| {
        coordinator_feedback::show(&mut draft, Some(&judgments), false, "en", ui);
    });
    let rendered = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for expected in [
        "Recipient: claude",
        "Coordinator request processing",
        "Send to coordinator",
    ] {
        assert!(
            rendered.contains(expected),
            "missing {expected}: {rendered}"
        );
    }
}

#[test]
fn coordinator_panel_renders_the_latest_request_lifecycle_state() {
    let context = egui::Context::default();
    context.enable_accesskit();
    for (status, expected) in [
        ("queued", "Coordinator request queued"),
        ("processing", "Coordinator request processing"),
        ("completed", "Coordinator request completed"),
        ("failed", "Coordinator request failed"),
    ] {
        let judgments = json!({"coordinator":"fixture-reviewer","decision_count":1,"judgments":[
            {"kind":"request","judgment":status,"detail":"Lifecycle detail"}
        ]});
        let mut draft = String::new();
        let mut output = context.run_ui(Default::default(), |ui| {
            coordinator_feedback::show(&mut draft, Some(&judgments), false, "en", ui);
        });
        let rendered = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ");
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        assert!(rendered.contains(expected), "missing {expected}: {rendered}");
    }
}
