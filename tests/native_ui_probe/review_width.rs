use super::{Probe, show};
use eframe::egui;
use serde_json::json;

#[test]
fn empty_plan_review_card_uses_the_available_content_width() {
    let context = egui::Context::default();
    let mut probe = Probe {
        snapshot: Some(json!({"proposal":{"plan":null}})),
        ..Default::default()
    };
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(860.0, 720.0),
            )),
            ..Default::default()
        },
        |ui| show(&mut probe, ui),
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let card_width = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.stroke.width > 0.0 => Some(rect.rect.width()),
            _ => None,
        })
        .max_by(f32::total_cmp)
        .unwrap_or_default();
    assert!(card_width >= 800.0, "Review card width was {card_width}px");
}

#[test]
fn a_previous_plan_failure_is_hidden_while_replanning_or_reporting_its_timeout() {
    let failed = json!({"job":{"kind":"plan","state":"failed"}});
    let preparing = Probe {
        pending: Some(super::super::Pending::Prepare),
        ..Default::default()
    };
    assert!(!super::plan_failure_is_current(&preparing, &failed));
    let refreshing = Probe {
        pending: Some(super::super::Pending::PrepareRefresh),
        ..Default::default()
    };
    assert!(!super::plan_failure_is_current(&refreshing, &failed));

    let timed_out = Probe {
        error: Some("native request timed out".into()),
        ..Default::default()
    };
    assert!(!super::plan_failure_is_current(&timed_out, &failed));
    assert!(super::can_retry_plan(&timed_out, &failed, true, false));
}

#[test]
fn a_failed_plan_can_be_retried_only_when_workspace_and_task_edits_are_saved() {
    let failed = json!({"job":{"kind":"plan","state":"failed"}});
    let idle = Probe::default();

    assert!(super::plan_failure_is_current(&idle, &failed));
    assert!(super::can_retry_plan(&idle, &failed, true, false));
    assert!(!super::can_retry_plan(&idle, &failed, false, false));
    assert!(!super::can_retry_plan(&idle, &failed, true, true));
    assert!(!super::can_retry_plan(
        &idle,
        &json!({"job":{"kind":"start","state":"failed"}}),
        true,
        false
    ));
}

#[test]
fn review_renders_a_retry_action_for_a_saved_failed_plan() {
    let context = egui::Context::default();
    let mut snapshot = json!({
        "job":{"kind":"plan","state":"failed","error":"previous attempt"},
        "proposal":{"plan":{"tasks":[]},"digest":""},
        "revision":"revision",
        "config":{"name":"","repository":"","goal":"","coordinator":"","profiles":[]}
    });
    snapshot["config"]["workspace_mode"] =
        serde_json::to_value(crate::plan::WorkspaceMode::default()).unwrap();
    snapshot["config"]["roles"] = serde_json::to_value(crate::roles::RoleRules::default()).unwrap();
    let mut probe = Probe {
        snapshot: Some(snapshot),
        ..Default::default()
    };
    assert!(super::settings_are_current(
        &probe,
        probe.snapshot.as_ref().unwrap()
    ));
    assert!(super::can_retry_plan(
        &probe,
        probe.snapshot.as_ref().unwrap(),
        true,
        false
    ));
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(860.0, 720.0),
            )),
            ..Default::default()
        },
        |ui| show(&mut probe, ui),
    );
    let retry_is_visible = output.shapes.iter().any(|shape| {
        matches!(
            &shape.shape,
            egui::Shape::Text(text) if text.galley.job.text == "계획 생성 다시 시도"
        )
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(retry_is_visible);
}
