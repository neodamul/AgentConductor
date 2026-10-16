use agentconductor::native_ui_probe::room::{self, State};
use eframe::egui;

#[test]
fn native_agent_room_controls_follow_the_selected_language() {
    let snapshot = serde_json::json!({
        "revision":"rev-1",
        "report":{
            "state":"running",
            "candidate":{"commit":"abc123"},
            "plan":{"goal":"Rotate tokens"}
        }
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let labels = |state: &mut State| {
        let mut output = context.run_ui(Default::default(), |ui| {
            room::show(state, Some(&snapshot), false, None, "ko", ui);
        });
        let labels = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter_map(|(_, node)| node.label().map(str::to_owned))
            .collect::<Vec<_>>();
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        labels
    };
    let initial = labels(&mut state);
    for label in [
        "실행 새로 고침",
        "읽기 전용 팀 브리핑 불러오기",
        "실행 중지",
        "변경 검토 근거 불러오기",
        "실행 재개",
        "중단된 실행 복구",
    ] {
        assert!(
            initial.iter().any(|value| value == label),
            "{label}; {initial:?}"
        );
    }
    state.confirming_stop = true;
    let confirming = labels(&mut state);
    for label in ["중지 사유", "중지 취소", "중지 확인"] {
        assert!(
            confirming.iter().any(|value| value == label),
            "{label}; {confirming:?}"
        );
    }
    state.confirming_recover = true;
    let confirming_recovery = labels(&mut state);
    for label in ["복구 취소", "복구 확인"] {
        assert!(
            confirming_recovery.iter().any(|value| value == label),
            "{label}; {confirming_recovery:?}"
        );
    }
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "ko", ui);
    });
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
        visible.contains("살아 있거나 알 수 없는 프로세스"),
        "{visible}"
    );
}
