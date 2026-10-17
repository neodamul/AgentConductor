use super::*;

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
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
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
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
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
        room::show(&mut state, Some(&completed), false, None, "en", ui);
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
fn native_room_candidate_decisions_require_current_evidence_and_confirmation() {
    use agentconductor::native_ui_probe::room::{CandidateDecision, State};
    use eframe::egui::accesskit::Role;

    let context = egui::Context::default();
    context.enable_accesskit();
    let snapshot = json!({
        "revision":"rev-current",
        "report":{"state":"waiting_approval","candidate":{"commit":"candidate-sha"}}
    });
    let mut state = State::default();
    state.evidence =
        Some(json!({"revision":"rev-current","report":{"candidate":{"commit":"candidate-sha"}}}));
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"Approve verified candidate"));
    assert!(labels.contains(&"Reject candidate"));
    assert!(!labels.contains(&"Confirm candidate decision"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    state.confirming_decision = Some(CandidateDecision::Approve);
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
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
    assert!(labels.contains(&"Confirm candidate decision"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    let changed = json!({
        "revision":"rev-new",
        "report":{"state":"approved","candidate":{"commit":"candidate-sha"}}
    });
    state.evidence =
        Some(json!({"revision":"rev-current","report":{"candidate":{"commit":"candidate-sha"}}}));
    state.confirming_decision = Some(CandidateDecision::Merge);
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&changed), false, None, "en", ui);
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
    assert!(!labels.contains(&"Confirm candidate decision"));
    assert_eq!(state.confirming_decision, None);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
