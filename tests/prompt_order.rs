//! GitHub #211: every prompt the application writes is assembled as Document, then Task, then Format
//! (docs/cli-io-protocol.md §2.2): the material first, what to do with it next, the shape of the answer last.
use agentconductor::{chat, planner, prompt_optimizer, roles::RoleRules};

/// Positions of the three block headings; each must appear exactly once, in this order.
fn blocks(prompt: &str) -> [usize; 3] {
    ["# Document\n", "# Task\n", "# Format\n"].map(|heading| {
        assert_eq!(
            prompt.matches(heading).count(),
            1,
            "{heading:?} must appear once"
        );
        prompt.find(heading).unwrap()
    })
}
fn in_order(prompt: &str) -> (&str, &str, &str) {
    let [d, t, f] = blocks(prompt);
    assert!(d < t && t < f, "Document, Task, Format");
    (&prompt[d..t], &prompt[t..f], &prompt[f..])
}

#[test]
fn the_chat_prompt_puts_the_manual_and_earlier_turns_first_and_the_answer_shape_last() {
    let turns: Vec<chat::Turn> = serde_json::from_value(serde_json::json!([
        {"role":"operator","text":"What was built?"},
        {"role":"assistant","text":"A ladder game."},
        {"role":"operator","text":"How do I run it?"}
    ]))
    .unwrap();
    let prompt = chat::build_prompt(&turns, "Korean").unwrap();
    let (document, task, format) = in_order(&prompt);
    assert!(
        document.contains("AgentConductor")
            && document.contains("What was built?")
            && document.contains("A ladder game.")
    );
    assert!(
        !document.contains("How do I run it?"),
        "the current question belongs to the task"
    );
    assert!(task.contains("read-only") && task.contains("How do I run it?"));
    assert!(format.contains("Reply in Korean") && format.contains("\"answer\""));
    assert!(
        !task.contains("Output one JSON") && !document.contains("Output one JSON"),
        "the shape of the answer is last"
    );
}

#[test]
fn the_optimizer_prompt_puts_the_skill_and_the_goal_first() {
    let prompt = prompt_optimizer::build_prompt("Make the checkout faster", "Korean").unwrap();
    let (document, task, format) = in_order(&prompt);
    assert!(
        document.contains("Make the checkout faster"),
        "the goal to rewrite is material"
    );
    assert!(
        document.contains("prompt-optimizer"),
        "the skill and its references are material"
    );
    assert!(task.contains("read-only") && task.contains("Rewrite"));
    assert!(format.contains("Reply in Korean") && format.contains("\"optimized\""));
}

#[test]
fn the_planning_prompt_puts_source_documents_first_and_the_json_contract_last() {
    let roles = RoleRules::new();
    let prompt = planner::plan_prompt(planner::PromptInput {
        goal: "Add a ladder game",
        agent_ids: "[\"claude\"]",
        capabilities: "[]",
        roles: &roles,
        rules: Some("Keep tests small."),
        phased: true,
        reference: Some("- [ ] unfinished item"),
        context: Some("Acceptance: three players."),
    });
    let (document, task, format) = in_order(&prompt);
    assert!(
        document.contains("unfinished item") && document.contains("Acceptance: three players.")
    );
    assert!(
        task.contains("Goal: Add a ladder game")
            && task.contains("Keep tests small.")
            && task.contains("Registered agent IDs")
            && task.contains("Task types")
    );
    assert!(task.contains("ordered phases"));
    assert!(
        format.contains("Return only a JSON object")
            && format.contains("\"phases\"")
            && format.contains("Done when:")
    );
    for early in [document, task] {
        assert!(
            !early.contains("Return only a JSON object"),
            "the contract is last"
        );
    }
    let plain = planner::plan_prompt(planner::PromptInput {
        goal: "g",
        agent_ids: "[]",
        capabilities: "[]",
        roles: &roles,
        rules: None,
        phased: false,
        reference: None,
        context: None,
    });
    let (document, _, _) = in_order(&plain);
    assert!(
        document.trim_end().ends_with("(none)"),
        "an empty Document says so: {document:?}"
    );
}
