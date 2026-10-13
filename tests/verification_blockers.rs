use agentconductor::workflow::blockers::blocking_findings;

#[test]
fn a_report_that_records_a_blocking_issue_is_found_and_non_blocking_notes_are_not() {
    // Shape of the real phase-01 verification report (2026-10-13).
    let report = "# 1페이즈 검증 보고서\n\n| 항목 | 판정 |\n| --- | --- |\n| resolveAll 공정성 | **실패 — 차단 이슈 1건** |\n\n\
**차단 이슈: 1건 (B-01).**\n\n## 5. 발견 사항\n\n### 차단\n\n- **B-01 사다리 공정성 미달.** `src/core/ladder.js:24-43`의 `generateLadder`는 편향됩니다.\n\n\
### 비차단\n\n- **N-01 골든 벡터 부재.**\n- **N-02 정수 시드 충돌.**\n\n## 6. 후속 조치\n\n- B-01 수정 후 재검증\n";
    let found = blocking_findings(report).expect("B-01 blocks");
    assert!(found.contains("B-01") && found.contains("src/core/ladder.js"));
    assert!(!found.contains("N-01") && !found.contains("재검증"));

    for clean in [
        "# Verification\n\n## Blocking issues\n\n- None\n\n## Non-blocking\n\n- small nit\n",
        "## 차단\n\n없음\n\n## 비차단\n\n- 표기 개선\n",
        "Blocking issues: 0\n",
        "# Report\n\nAll tests pass.\n",
    ] {
        assert_eq!(blocking_findings(clean), None, "{clean}");
    }
    assert_eq!(
        blocking_findings("Summary\n\nBlocking issues: 2 (see tracker)\n").as_deref(),
        Some("Blocking issues: 2 (see tracker)")
    );
    assert!(blocking_findings("## Blockers\n\n1. Race in scheduler\n").is_some());
}
