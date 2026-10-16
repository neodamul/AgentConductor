use agentconductor::native_ui_probe::{
    CandidateDecision, Probe, RunAction, Surface, ToolAction, candidate_decision_request,
    run_action_request, tool_action_request,
};
use eframe::egui;
use std::{fs, sync::Arc, time::Duration};

#[path = "support/workflow.rs"]
mod support;

fn temp_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "agentconductor-native-surface-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn workspace_launch_surface_connects_to_workspace_controls() {
    let root = temp_dir();
    let room = Arc::new(
        agentconductor::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap(),
    );
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe =
        Probe::connect_surface(room.clone(), context.clone(), Surface::Workspace).unwrap();
    let mut output = context.run_ui(Default::default(), |ui| probe.show(ui));
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"Workspace name"));
    assert!(!labels.contains(&"Agent Room"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    drop(output);
    drop(probe);
    drop(room);
    std::thread::sleep(Duration::from_millis(20));
    fs::remove_dir_all(root).unwrap();
}

fn candidate(state: &str) -> serde_json::Value {
    serde_json::json!({
        "revision":"run-rev-1",
        "report":{"state":state,"candidate":{"commit":"candidate-sha"}}
    })
}

#[test]
fn approve_request_uses_the_exact_candidate_and_revision() {
    let packet = candidate_decision_request(
        &candidate("waiting_approval"),
        CandidateDecision::Approve,
        "",
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["revision"], "run-rev-1");
    assert_eq!(
        value["action"],
        serde_json::json!({"kind":"approve","sha":"candidate-sha"})
    );
}

#[test]
fn merge_request_is_refused_until_the_candidate_is_approved() {
    assert!(
        candidate_decision_request(&candidate("waiting_approval"), CandidateDecision::Merge, "")
            .is_err()
    );
    let packet =
        candidate_decision_request(&candidate("approved"), CandidateDecision::Merge, "").unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["action"]["kind"], "merge");
    assert_eq!(value["action"]["sha"], "candidate-sha");
}

#[test]
fn reject_request_requires_a_bounded_reason_and_current_candidate() {
    assert!(
        candidate_decision_request(
            &candidate("waiting_approval"),
            CandidateDecision::Reject,
            " "
        )
        .is_err()
    );
    assert!(
        candidate_decision_request(
            &candidate("waiting_approval"),
            CandidateDecision::Reject,
            &"x".repeat(16 * 1024 + 1)
        )
        .is_err()
    );
    let packet = candidate_decision_request(
        &candidate("waiting_approval"),
        CandidateDecision::Reject,
        "missing edge case",
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(
        value["action"],
        serde_json::json!({"kind":"reject","reason":"missing edge case"})
    );
}

#[test]
fn resume_and_recover_requests_are_revision_bound_and_state_checked() {
    let packet = run_action_request(&candidate("stopped"), RunAction::Resume).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["revision"], "run-rev-1");
    assert_eq!(value["action"]["kind"], "resume");
    assert!(run_action_request(&candidate("waiting_approval"), RunAction::Resume).is_err());
    assert!(run_action_request(&candidate("completed"), RunAction::Resume).is_err());
    let packet = run_action_request(&candidate("running"), RunAction::Recover).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["action"]["kind"], "recover");
    assert!(run_action_request(&candidate("stopped"), RunAction::Recover).is_err());
}

#[test]
fn tool_decision_is_bound_to_a_current_pending_receipt() {
    let id = "a".repeat(64);
    let mut snapshot = candidate("running");
    snapshot["tool_requests"] = serde_json::json!([{
        "id":id,
        "status":"pending",
        "request":{"tool":"Bash","input":{"command":"make test"}},
        "context":{"task":"test-suite"}
    }]);
    let packet = tool_action_request(&snapshot, &id, ToolAction::Approve).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["revision"], "run-rev-1");
    assert_eq!(
        value["action"],
        serde_json::json!({"kind":"tool_approve","sha":id})
    );
    snapshot["tool_requests"][0]["status"] = "approved".into();
    let packet = tool_action_request(&snapshot, &id, ToolAction::Refresh).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["action"]["kind"], "tool_refresh");
    assert!(tool_action_request(&snapshot, &id, ToolAction::Approve).is_err());
    assert!(tool_action_request(&snapshot, &"b".repeat(64), ToolAction::Reject).is_err());
    snapshot["tool_requests"][0]["status"] = "expired".into();
    assert!(tool_action_request(&snapshot, &id, ToolAction::Refresh).is_err());
}

#[test]
fn native_tool_panel_shows_receipt_and_drops_confirmation_when_stale() {
    let id = "c".repeat(64);
    let mut snapshot = serde_json::json!({
        "revision":"rev-1",
        "report":{"state":"running","candidate":null},
        "tool_requests":[{
            "id":id,
            "status":"pending",
            "request":{"tool":"Bash","input":{"command":"make test"},"reason":"Run integration tests"},
            "context":{"task":"test-suite"}
        }]
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = agentconductor::native_ui_probe::room::State::default();
    let labels = |output: &egui::FullOutput| {
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter_map(|(_, node)| node.label().map(str::to_owned))
            .collect::<Vec<_>>()
    };
    let rendered = |output: &egui::FullOutput| {
        output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::room::show(&mut state, Some(&snapshot), false, None, ui);
    });
    let text = rendered(&output);
    let buttons = labels(&output);
    let has_tool = text.contains("Bash · pending");
    let has_task = text.contains("Task · test-suite");
    let no_confirmation = !buttons.contains(&"Confirm tool request decision".to_owned());
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(has_tool, "{text}");
    assert!(has_task, "{text}");
    assert!(no_confirmation, "{buttons:?}");

    state.confirming_tool = Some((id.clone(), ToolAction::Approve, "rev-1".into()));
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::room::show(&mut state, Some(&snapshot), false, None, ui);
    });
    let confirmation_shown = labels(&output).contains(&"Confirm tool request decision".to_owned());
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(confirmation_shown);

    snapshot["revision"] = "rev-2".into();
    snapshot["tool_requests"][0]["status"] = "approved".into();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::room::show(&mut state, Some(&snapshot), false, None, ui);
    });
    let confirmation_hidden =
        !labels(&output).contains(&"Confirm tool request decision".to_owned());
    let confirmation_cleared = state.confirming_tool.is_none();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(confirmation_hidden);
    assert!(confirmation_cleared);

    let approved_actions = labels(&output);
    assert!(approved_actions.contains(&format!("Refresh request tool request {id}")));
    assert!(!approved_actions.contains(&format!("Allow once tool request {id}")));
}

#[test]
fn native_room_renders_read_only_team_briefing() {
    let context = egui::Context::default();
    let mut state = agentconductor::native_ui_probe::room::State::default();
    state.briefing = Some(serde_json::json!({
        "state":"observed",
        "observed_at_ms":1_797_120_000_000_u64,
        "sessions":[{
            "agent":"Claude",
            "task":"Implement authentication",
            "state":"running",
            "work":"Rotate refresh tokens",
            "model":"Sonnet",
            "attempt":2,
            "session_id":"session-42",
            "process":{"state":"running","observed_at_ms":1_797_120_000_000_u64},
            "last_activity":{"kind":"checkpoint","freshness":"fresh","text":"Tests are running"},
            "restriction":{"available_at":1_797_120_600_i64},
            "attention":"Waiting for integration test"
        }],
        "approvals":[{"kind":"tool","task":"auth-tests","status":"pending"}]
    }));
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::room::show(&mut state, None, false, None, ui);
    });
    let text = output
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
        text.contains("Claude · Implement authentication · running"),
        "{text}"
    );
    assert!(text.contains("Rotate refresh tokens"), "{text}");
    assert!(text.contains("Tests are running"), "{text}");
    assert!(text.contains("Waiting for integration test"), "{text}");
    assert!(text.contains("tool · auth-tests · pending"), "{text}");
}

#[tokio::test]
async fn native_worker_fetches_team_briefing_through_the_rust_dispatcher() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    agentconductor::workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    let room =
        Arc::new(agentconductor::ui::Room::open(&run, &std::env::current_exe().unwrap()).unwrap());
    let worker =
        agentconductor::native_ui_probe::RequestWorker::new(room.clone(), egui::Context::default())
            .unwrap();
    worker.send(br#"{"kind":"briefing"}"#).unwrap();
    let briefing = worker
        .replies
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
        .unwrap();
    assert!(briefing["state"].is_string());
    assert_eq!(briefing["sessions"].as_array().unwrap().len(), 3);
    assert!(briefing["approvals"].is_array());
    drop(worker);
    drop(room);
    std::thread::sleep(Duration::from_millis(20));
    fs::remove_dir_all(root).unwrap();
}
