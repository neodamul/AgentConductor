#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::{self, CandidateDecision, State};
use eframe::egui;
use serde_json::json;

fn labels(state: &mut State, snapshot: &serde_json::Value) -> Vec<String> {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(state, Some(snapshot), false, None, "ko", ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    labels
}

#[test]
fn candidate_decision_controls_and_confirmations_are_localized() {
    let snapshot = json!({
        "revision":"rev-1",
        "report":{
            "state":"waiting_approval",
            "candidate":{"commit":"abc123"},
            "plan":{"goal":"Rotate tokens"}
        }
    });
    let mut state = State::default();
    state.evidence = Some(json!({"revision":"rev-1"}));
    let initial = labels(&mut state, &snapshot);
    for label in ["검증된 후보 승인", "후보 반려", "승인된 후보 병합"] {
        assert!(
            initial.iter().any(|value| value == label),
            "{label}; {initial:?}"
        );
    }

    let approved_snapshot = json!({
        "revision":"rev-1",
        "report":{
            "state":"approved",
            "candidate":{"commit":"abc123"},
            "plan":{"goal":"Rotate tokens"}
        }
    });
    for (decision, snapshot, label) in [
        (
            CandidateDecision::Approve,
            &snapshot,
            "이 검증된 변경 사항과 테스트를 승인할까요?",
        ),
        (
            CandidateDecision::Merge,
            &approved_snapshot,
            "승인된 후보를 정확히 통합할까요?",
        ),
    ] {
        state.confirming_decision = Some(decision);
        let confirmation = labels(&mut state, snapshot);
        assert!(
            confirmation.iter().any(|value| value == label),
            "{label}; {confirmation:?}"
        );
    }

    state.confirming_decision = Some(CandidateDecision::Reject);
    let rejecting = labels(&mut state, &snapshot);
    for label in [
        "이 후보를 반려하고 피드백을 보내시겠습니까?",
        "거절 사유",
        "후보 결정 취소",
        "후보 결정 확인",
    ] {
        assert!(
            rejecting.iter().any(|value| value == label),
            "{label}; {rejecting:?}"
        );
    }
}
