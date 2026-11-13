use agentconductor::prompt_optimizer::{MAX_GOAL_BYTES, build_prompt_with_answers};
#[test]
fn answers_are_bounded_and_join_the_goal_in_the_document_before_the_task() {
    let prompt = build_prompt_with_answers(
        "Add rotation",
        "English",
        r#"{"questions":["Which environment?"],"answers":"Windows"}"#,
    )
    .unwrap();
    let start = prompt.find("## Operator clarification answers").unwrap();
    let task = prompt.rfind("# Task").unwrap();
    assert!(start < task && prompt[start..task].contains("Windows"));
    assert!(prompt.contains("Add rotation"));
    assert!(build_prompt_with_answers("goal", "English", &"a".repeat(MAX_GOAL_BYTES + 1)).is_err());
    assert!(build_prompt_with_answers("goal", "English", "").is_ok());
    assert!(
        build_prompt_with_answers(&"x".repeat(16 * 1024), "English", "Use Windows").is_ok(),
        "a bounded optimizer proposal can be clarified without truncating it"
    );
}
