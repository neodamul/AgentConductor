#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::header;
use eframe::egui::{self, Pos2, accesskit::Role};

fn rendered_labels(output: &egui::FullOutput) -> Vec<String> {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect()
}

#[test]
fn workspace_title_exposes_phase_and_new_goal_action_in_english_and_korean() {
    for (language, expected) in [
        (
            "en",
            ["What should the team work on?", "Prepared", "New goal"],
        ),
        ("ko", ["팀이 무엇을 해야 하나요?", "준비됨", "새 목표"]),
    ] {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    egui::vec2(1280.0, 180.0),
                )),
                ..Default::default()
            },
            |ui| {
                header::show_title_controls(0, language, "startup.prepared", true, ui);
            },
        );
        let labels = rendered_labels(&output);
        let has_button = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Button && node.label() == Some(expected[2]));
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        for label in expected {
            assert!(
                labels.iter().any(|item| item == label),
                "missing {label:?} in {language}; labels: {labels:?}"
            );
        }
        assert!(
            has_button,
            "new-goal action must be an accessible button in {language}"
        );
    }
}

#[test]
fn new_goal_action_reports_an_enabled_pointer_click() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let screen = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 120.0));
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        },
        |ui| {
            header::show_title_controls(0, "en", "startup.ready", true, ui);
        },
    );
    let position = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some("New goal"))
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| {
            Pos2::new(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            )
        })
        .expect("new-goal button bounds");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let mut clicked = false;
    output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen),
            events: vec![
                egui::Event::PointerMoved(position),
                egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            ..Default::default()
        },
        |ui| clicked = header::show_title_controls(0, "en", "startup.ready", true, ui),
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        clicked,
        "enabled new-goal click must be returned to its caller"
    );
}

#[test]
fn header_status_tracks_job_proposal_and_unsaved_workspace_state() {
    let mut snapshot = serde_json::json!({"proposal":{"phase":"ready"}});
    assert_eq!(header::workspace_phase(&snapshot, false), "startup.ready");
    snapshot["job"] = serde_json::json!({"kind":"plan","state":"running"});
    assert_eq!(
        header::workspace_phase(&snapshot, true),
        "startup.preparing"
    );
    snapshot["job"] = serde_json::json!({"kind":"start","state":"running"});
    assert_eq!(
        header::workspace_phase(&snapshot, false),
        "startup.starting"
    );
    snapshot["job"]["state"] = serde_json::json!("idle");
    snapshot["proposal"] = serde_json::Value::Null;
    assert_eq!(header::workspace_phase(&snapshot, true), "startup.unsaved");
    assert_eq!(
        header::workspace_phase(&snapshot, false),
        "startup.prepared"
    );
}

#[test]
fn new_goal_request_is_revisioned_and_disabled_when_workspace_is_not_idle() {
    let snapshot = serde_json::json!({
        "revision":"r17",
        "config":{"repository":"/repo"},
        "job":{"state":"idle"},
        "run_state":"completed"
    });
    assert_eq!(
        goals_request(&snapshot),
        br#"{"kind":"new_goal","revision":"r17"}"#
    );
    assert!(
        agentconductor::native_ui_probe::workspace::goals::can_start_new_goal(
            &snapshot, false, false
        )
    );
    assert!(
        !agentconductor::native_ui_probe::workspace::goals::can_start_new_goal(
            &snapshot, true, false
        )
    );
    assert!(
        !agentconductor::native_ui_probe::workspace::goals::can_start_new_goal(
            &snapshot, false, true
        )
    );
}

fn goals_request(snapshot: &serde_json::Value) -> Vec<u8> {
    agentconductor::native_ui_probe::workspace::goals::new_goal_request(
        snapshot["revision"].as_str().unwrap(),
    )
}
