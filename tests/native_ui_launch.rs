use agentconductor::native_ui_probe::{
    CandidateDecision, Probe, Surface, candidate_decision_request,
};
use eframe::egui;
use std::{fs, sync::Arc, time::Duration};

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
