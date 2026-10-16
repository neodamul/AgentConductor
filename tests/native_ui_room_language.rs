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
    assert!(visible.contains("에이전트 작업실"), "{visible}");
    assert!(
        visible.contains("살아 있거나 알 수 없는 프로세스"),
        "{visible}"
    );
}

#[test]
fn native_team_briefing_content_uses_the_selected_language() {
    let snapshot = serde_json::json!({
        "revision":"rev-1",
        "report":{"state":"running","candidate":null}
    });
    let mut state = State::default();
    state.briefing = Some(serde_json::json!({
        "state":"completed",
        "observed_at_ms":1_700_000_000_000i64,
        "sessions":[{
            "agent":"codex",
            "task":"health-tests",
            "state":"waiting_availability",
            "work":"Writing offline tests",
            "session_id":"session-1",
            "model":"gpt-6",
            "attempt":2,
            "process":{"state":"completed","observed_at_ms":1_700_000_000_000i64},
            "last_activity":{"kind":"report","freshness":"fresh","text":"Tests pass","observed_at_ms":1_700_000_000_000i64},
            "restriction":{"available_at":1_700_000_000_000i64},
            "attention":"Review the result",
            "next_action":"Inspect the diff"
        }],
        "approvals":[]
    }));
    let context = egui::Context::default();
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
    for label in [
        "팀 브리핑 · 읽기 전용",
        "완료",
        "세션: session-1 · 모델: gpt-6 · 시도 2",
        "사용 한도 대기",
        "프로세스 상태: 완료",
        "기록된 승인 요청이 없습니다.",
    ] {
        assert!(visible.contains(label), "missing {label}; {visible}");
    }
}

#[test]
fn native_review_evidence_content_uses_the_selected_language() {
    let evidence = serde_json::json!({
        "revision":"review-7",
        "truncated":true,
        "report":{
            "candidate":{"commit":"abc123"},
            "tests":[{"task":"unit","exit_code":124,"timed_out":true}]
        },
        "test_output":[{"task":"unit","stdout":"passed","stderr":""}],
        "diff":"+localized review"
    });
    let context = egui::Context::default();
    let mut diff = evidence["diff"].as_str().unwrap().to_owned();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::render_evidence_in_language(
            &evidence, &mut diff, "ko", ui,
        );
    });
    let visible = output
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
    for label in [
        "검토 증거",
        "증거 리비전 · review-7",
        "후보 · abc123",
        "테스트 · unit · 종료 코드 124 · 시간 초과",
        "unit · 표준 출력",
        "Diff가 512 KiB에서 잘렸습니다.",
        "passed",
        "+localized review",
    ] {
        assert!(visible.contains(label), "missing {label}; {visible}");
    }
}
