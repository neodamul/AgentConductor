use eframe::egui;
use serde_json::json;

#[test]
fn native_room_snapshot_shows_goal_and_agent_task_statuses() {
    let snapshot = json!({
        "state": "running",
        "plan": {
            "goal": "Rotate refresh tokens",
            "tasks": [
                {"id": "implementation", "agent": "claude"},
                {"id": "integration-tests", "agent": "codex"}
            ]
        },
        "tasks": [
            {"id": "implementation", "status": "running"},
            {"id": "integration-tests", "status": "pending"}
        ]
    });
    let context = egui::Context::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::Probe::render_snapshot(&snapshot, ui);
    });
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let text = text.join(" ");
    assert!(text.contains("Rotate refresh tokens"));
    assert!(text.contains("implementation"));
    assert!(text.contains("claude"));
    assert!(text.contains("running"));
    assert!(text.contains("integration-tests"));
    assert!(text.contains("pending"));
}

#[test]
fn native_room_snapshot_reads_the_real_controller_report_envelope() {
    let snapshot = json!({
        "report": {
            "state": "running",
            "plan": {
                "goal": "Rotate refresh tokens",
                "tasks": [{"id": "integration-tests", "agent": "codex"}]
            },
            "tasks": [{"id": "integration-tests", "status": "running"}]
        }
    });
    let context = egui::Context::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::Probe::render_snapshot(&snapshot, ui);
    });
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(text.contains("Rotate refresh tokens"));
    assert!(text.contains("Run · running"));
    assert!(text.contains("integration-tests"));
    assert!(text.contains("codex"));
}

#[test]
fn native_workspace_review_renders_the_proposed_plan() {
    let proposal = json!({
        "proposal": {
            "phase": "ready",
            "plan": {
                "goal": "Build the native workspace flow",
                "tasks": [{"id": "native-screen", "agent": "codex"}]
            }
        }
    });
    let context = egui::Context::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::Probe::render_snapshot(&proposal, ui);
    });
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    assert!(text.contains("Build the native workspace flow"));
    assert!(text.contains("native-screen"));
    assert!(text.contains("codex"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn native_agent_status_colors_distinguish_active_waiting_and_failed() {
    use agentconductor::native_ui_probe::status_color;
    use egui::Color32;

    assert_eq!(
        status_color("waiting_approval", 0.0),
        Color32::from_rgb(218, 148, 34)
    );
    assert_eq!(status_color("failed", 0.0), Color32::from_rgb(205, 55, 55));
    assert_ne!(status_color("running", 0.0), status_color("running", 0.5));
}

#[test]
fn native_room_refresh_matches_the_existing_active_and_idle_poll_cadence() {
    use std::time::Duration;

    assert_eq!(
        agentconductor::native_ui_probe::room_refresh_interval(&json!({
            "report": {"state": "running"}
        })),
        Duration::from_secs(1)
    );
    assert_eq!(
        agentconductor::native_ui_probe::room_refresh_interval(&json!({
            "report": {"state": "completed"}
        })),
        Duration::from_secs(5)
    );
}

#[test]
fn native_room_poll_shortens_an_idle_deadline_when_work_becomes_active() {
    use std::time::{Duration, Instant};

    let mut next = Some(Instant::now() + Duration::from_secs(5));
    let snapshot = json!({"report": {"state": "running"}});
    assert!(!agentconductor::native_ui_probe::refresh_due(
        &egui::Context::default(),
        Some(&snapshot),
        false,
        &mut next,
    ));
    assert!(next.unwrap() <= Instant::now() + Duration::from_secs(2));
}

#[test]
fn native_room_stop_requires_a_confirmed_reason_and_is_hidden_when_terminal() {
    let snapshot = json!({
        "revision": "rev-1",
        "report": {
            "state": "waiting_availability",
            "plan": {"goal": "Stop only after review"}
        }
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = agentconductor::native_ui_probe::room::State::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::room::show(&mut state, Some(&snapshot), false, None, ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"Stop run"));
    assert!(!labels.contains(&"Confirm stop"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    state.confirming_stop = true;
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::room::show(&mut state, Some(&snapshot), false, None, ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"Stop reason"));
    assert!(labels.contains(&"Confirm stop"));
    assert!(labels.contains(&"Cancel stop"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    state.confirming_stop = true;
    let completed = json!({"report": {"state": "completed"}});
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::room::show(&mut state, Some(&completed), false, None, ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    assert!(!labels.contains(&"Stop run"));
    assert!(!labels.contains(&"Stop reason"));
    assert!(!state.confirming_stop);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn native_room_stop_packet_requires_a_nonempty_reason_and_current_revision() {
    let snapshot = json!({"revision": "rev-1", "report": {"state": "running"}});
    assert!(agentconductor::native_ui_probe::room::stop_request(&snapshot, " ").is_err());
    assert!(
        agentconductor::native_ui_probe::room::stop_request(&snapshot, &"x".repeat(16 * 1024 + 1))
            .is_err()
    );
    let packet =
        agentconductor::native_ui_probe::room::stop_request(&snapshot, "Operator requested stop")
            .unwrap();
    let packet: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(packet["kind"], "action");
    assert_eq!(packet["revision"], "rev-1");
    assert_eq!(packet["action"]["kind"], "stop");
    assert_eq!(packet["action"]["reason"], "Operator requested stop");
}

#[test]
fn native_room_offers_review_evidence_for_a_candidate() {
    let snapshot = json!({
        "report": {
            "state": "waiting_approval",
            "candidate": {"commit": "candidate-commit"}
        }
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = agentconductor::native_ui_probe::room::State::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::room::show(&mut state, Some(&snapshot), false, None, ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"Load review evidence"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn native_room_evidence_renders_candidate_tests_and_the_validated_diff() {
    let evidence = json!({
        "revision": "evidence-revision",
        "truncated": false,
        "report": {
            "candidate": {"commit": "candidate-commit"},
            "tests": [{"task": "integration", "exit_code": 0, "timed_out": false}]
        },
        "test_output": [{"task": "integration", "stdout": "3 tests passed", "stderr": ""}],
        "diff": "+refresh token rotation\n"
    });
    let context = egui::Context::default();
    let mut diff = evidence["diff"].as_str().unwrap().to_owned();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::render_evidence(&evidence, &mut diff, ui);
    });
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    assert!(text.contains("evidence-revision"));
    assert!(text.contains("candidate-commit"));
    assert!(text.contains("integration"));
    assert!(text.contains("3 tests passed"));
    assert!(text.contains("+refresh token rotation"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn native_room_evidence_rejects_stale_revisions_and_uses_a_read_only_request() {
    use agentconductor::native_ui_probe::room::{State, evidence_reply, evidence_request};

    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(evidence_request()).unwrap()["kind"],
        "evidence"
    );
    let mut state = State::default();
    let current = json!({"revision": "current"});
    let stale = json!({"revision": "old", "diff": "stale"});
    assert!(evidence_reply(&mut state, Some(&current), stale).is_err());
    assert!(state.evidence.is_none());
    assert!(state.diff.is_empty());
}
