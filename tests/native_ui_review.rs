use agentconductor::ui::messages::{in_language, interpolate};

#[test]
fn plan_review_copy_is_localized_in_english_and_korean() {
    let copy = [
        (
            "startup.review_connecting",
            "Connecting to the Rust workspace controller…",
            "Rust 워크스페이스 컨트롤러에 연결 중…",
        ),
        (
            "startup.review_coordinator_activity",
            "Coordinator activity: running",
            "코디네이터 활동: 실행 중",
        ),
        (
            "startup.review_planning_in_progress",
            "Planning in progress · coordinator {value1}",
            "계획 생성 중 · 코디네이터 {value1}",
        ),
        (
            "startup.review_elapsed",
            "Elapsed · {value1}s",
            "경과 시간 · {value1}초",
        ),
        (
            "startup.review_output_bytes",
            "Coordinator output · {value1} bytes",
            "코디네이터 출력 · {value1}바이트",
        ),
        (
            "startup.review_plan_generation_failed",
            "Plan generation failed",
            "계획 생성 실패",
        ),
        ("startup.review_logs", "Logs · {value1}", "로그 · {value1}"),
        ("startup.review_generate_plan", "Generate plan", "계획 생성"),
        (
            "startup.review_save_before_generate",
            "Save Workspace settings and Agent roles before generating the plan.",
            "계획을 생성하기 전에 워크스페이스 설정과 에이전트 역할을 저장하세요.",
        ),
    ];

    for (id, english, korean) in copy {
        assert_eq!(in_language("en", id), english, "English: {id}");
        assert_eq!(in_language("ko", id), korean, "Korean: {id}");
    }
    assert_eq!(
        interpolate(
            in_language("ko", "startup.review_planning_in_progress"),
            &[("value1", "codex")]
        ),
        "계획 생성 중 · 코디네이터 codex"
    );
}
