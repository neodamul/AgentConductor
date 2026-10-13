use agentconductor::slug_progress::briefing;
use std::fs;

#[test]
fn every_phase_is_briefed_with_its_planned_tasks_for_review() {
    let root = std::env::temp_dir().join(format!("ac-goal-briefing-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("plan")).unwrap();
    fs::create_dir_all(root.join("phase")).unwrap();
    // Shape of the coordinator's analysis for the ladder goal (2026-10-13).
    fs::write(
        root.join("plan/analysis.md"),
        "## 목표 분석\n- 사다리 게임\n\n## 페이즈 구성 (각 페이즈: 작업 -> 검증 -> 완료)\n\
         1. **01-ladder-core**: 요구사항 추적 문서, 시드 RNG, 사다리 로직(TDD).\n\
         2. **02-race-core**: 구슬 레이스 우선순위 로직(TDD). 1페이즈의 RNG를 재사용합니다.\n\
         3. **03-web-ui**: 두 게임의 브라우저 UI.\n\n## 위험 요소와 결정\n- 공정성\n",
    )
    .unwrap();
    fs::write(
        root.join("phase/04-release.md"),
        "# Phase 4 — release\n\nTasks:\n- t11-readme\n",
    )
    .unwrap();
    let spec = |id: &str| serde_json::json!({"id":id,"agent":"codex","model":null,"paths":["src"],"depends_on":[],"test_command":["node","--test"],"prompt":format!("Goal:\n- do {id}")});
    let plan = serde_json::json!({"phases":[
        {"id":"01-ladder-core","name":"ladder core","tasks":["t01"],"specs":[spec("t01")]},
        {"id":"02-race-core","name":"marble race core","tasks":["t05","t06"],"specs":[spec("t05"),spec("t06")]},
        {"id":"04-release","name":"release","tasks":["t11-readme"],"specs":[spec("t11-readme")]},
    ]});
    fs::write(root.join("plan/plan.json"), plan.to_string()).unwrap();

    let brief = briefing(&root);
    let phases = brief["phases"].as_array().unwrap();
    assert_eq!(phases.len(), 3);
    assert_eq!(
        phases[1]["brief"],
        "2. 02-race-core: 구슬 레이스 우선순위 로직(TDD). 1페이즈의 RNG를 재사용합니다."
    );
    assert_eq!(phases[1]["specs"].as_array().unwrap().len(), 2);
    assert_eq!(phases[1]["specs"][0]["id"], "t05");
    assert!(
        phases[1]["specs"][0]["prompt"]
            .as_str()
            .unwrap()
            .contains("do t05")
    );
    // Without an analysis item, the phase file describes the phase.
    assert!(phases[2]["brief"].as_str().unwrap().contains("t11-readme"));
    assert!(
        brief["analysis"]
            .as_str()
            .unwrap()
            .starts_with("## 목표 분석")
    );
    fs::remove_dir_all(root).unwrap();
}
