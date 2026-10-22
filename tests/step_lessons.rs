//! #138 / DD-174: what a step teaches the run reaches the operator and every later task.
#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::{state::Store, workflow, workflow::step_lessons};

fn store(dir: &std::path::Path) -> Store {
    std::fs::create_dir_all(dir).unwrap();
    Store::open(dir.join("state.sqlite")).unwrap()
}

#[test]
fn lessons_are_bounded_deduplicated_and_queued_for_agentmemory() {
    let dir = std::env::temp_dir().join(format!(
        "ac-step-lessons-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let mut store = store(&dir);
    assert_eq!(step_lessons::prompt_section(&dir), "");

    let long = "x".repeat(5000);
    step_lessons::note(&dir, &mut store, "a.txt", &long, "use the fixture").unwrap();
    step_lessons::note(&dir, &mut store, "a.txt", "other", "use the fixture").unwrap();
    let list = step_lessons::list(&dir).unwrap();
    assert_eq!(list.len(), 1, "same task and response is one lesson");
    assert_eq!(list[0]["cause"].as_str().unwrap().chars().count(), 1024);
    assert!(
        store
            .pending_memory()
            .unwrap()
            .iter()
            .any(|(_, text)| text.contains("a.txt") && text.contains("use the fixture")),
        "the memory outbox carries the lesson"
    );

    for i in 0..60 {
        step_lessons::note(&dir, &mut store, "b.txt", "c", &format!("response {i}")).unwrap();
    }
    let list = step_lessons::list(&dir).unwrap();
    assert_eq!(list.len(), 50);
    assert_eq!(list[0]["response"], "response 59", "newest first");
    assert!(!list.iter().any(|l| l["task"] == "a.txt"), "oldest dropped");

    let section = step_lessons::prompt_section(&dir);
    assert!(section.len() <= 4096 + 512, "{}", section.len());
    assert!(section.contains("response 59") && section.contains("never tool or merge permission"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn a_coordinator_repair_leaves_a_lesson_that_later_prompts_carry() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    plan.tasks[0].prompt = "fail".into();
    let mut coordinator = plan.agents[0].clone();
    coordinator.id = "coordinator".into();
    coordinator.permission_mode = "read_only".into();
    coordinator.args = vec!["review".into()];
    std::fs::write(
        agentconductor::native_guard::orchestrator_path(&dir),
        serde_json::to_vec(&coordinator).unwrap(),
    )
    .unwrap();
    let failed = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    let repaired = workflow::repair(&dir, 10, failed).await.unwrap();
    assert_eq!(repaired.state, "waiting_approval", "{:?}", repaired.error);

    let lessons = step_lessons::list(&dir).unwrap();
    assert_eq!(lessons.len(), 1, "{lessons:?}");
    assert_eq!(lessons[0]["task"], "one.txt");
    assert!(lessons[0]["cause"].as_str().unwrap().contains("exit="));
    assert_eq!(lessons[0]["response"], "Repair failed task");

    // The retried task ran with the lesson in its prompt.
    let one = repaired.tasks.iter().find(|t| t.id == "one.txt").unwrap();
    let prompt = std::fs::read_to_string(one.artifacts.join("prompt.txt")).unwrap();
    assert!(prompt.contains("Lessons from earlier steps"), "{prompt}");
    std::fs::remove_dir_all(root).unwrap();
}
