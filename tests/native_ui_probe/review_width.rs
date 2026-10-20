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
fn reviewed_plan_discloses_its_snapshot_base_and_execution_checks() {
    let base = "0123456789abcdef";
    let snapshot = json!({
        "revision":"revision",
        "config":{"name":"","goal":"Goal","repository":"/repo","coordinator":"","workspace_mode":"shared","profiles":[],"roles":{}},
        "proposal":{"phase":"ready","digest":"digest","repository_base":base,
            "plan":{"goal":"Goal","workspace_mode":"shared","agents":[],"tasks":[]}}
    });
    let mut probe = Probe {
        snapshot: Some(snapshot),
        ..Default::default()
    };
    let language = probe.navigation.appearance.language().to_owned();
    let base_prefix: String = base.chars().take(12).collect();
    let expected = crate::ui::messages::interpolate(
        crate::ui::messages::in_language(
            &language,
            "startup.snapshot_base_review_every_executable_permission_mode_scope",
        ),
        &[("value1", base_prefix.as_str())],
    );
    let context = egui::Context::default();
    context.enable_accesskit();
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
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(labels.contains(&expected.as_str()), "{labels:?}");
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
    let retry_label = probe
        .navigation
        .appearance
        .text("startup.retry_plan_generation");
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
            egui::Shape::Text(text) if text.galley.job.text == retry_label
        )
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(retry_is_visible);
}

#[test]
fn an_already_started_plan_is_not_presented_as_pending_after_returning_to_workspace() {
    let context = egui::Context::default();
    context.enable_accesskit();
    context.memory_mut(|memory| memory.set_everything_is_visible(true));
    let mut probe = Probe {
        snapshot: Some(json!({
            "run_state":"completed",
            "proposal":{"phase":"starting","digest":"digest","plan":{
                "goal":"Goal","workspace_mode":"shared","agents":[],
                "tasks":[{"id":"task-0","agent":"codex","prompt":"Implement","paths":[],"depends_on":[],"test_command":[]}]
            }}
        })),
        ..Default::default()
    };
    let expected_message = probe
        .navigation
        .appearance
        .text("startup.review_plan_already_started")
        .to_owned();
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
    let visible_text = output
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
    assert!(
        visible_text.contains(expected_message.as_str()),
        "{visible_text}"
    );
    assert!(!visible_text.contains("task-0"));
}
