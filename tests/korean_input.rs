#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;

const GOAL: &str = "세 파일을 만든다: 한글 목표와 피드백이 그대로 전달되어야 한다 (QAS-25)";

#[tokio::test]
async fn korean_goal_round_trips_into_the_planning_prompt_and_plan() {
    let (root, plan) = support::setup("shared");
    let mut profiles = plan.agents.clone();
    profiles[0].args = vec!["plan".into()];
    profiles[0].permission_mode = "read_only".into();
    let result = agentconductor::planner::generate(
        GOAL,
        &profiles,
        "mock",
        &root.join("repo"),
        &root.join("planning"),
    )
    .await
    .unwrap();
    assert_eq!(result.goal, GOAL);
    let prompt = std::fs::read_to_string(root.join("planning/prompt.txt")).unwrap();
    assert!(prompt.contains(GOAL));
    std::fs::remove_dir_all(root).unwrap();
}
