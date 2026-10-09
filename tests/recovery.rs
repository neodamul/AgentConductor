#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::{availability, workflow};

#[tokio::test]
async fn profile_concurrency_limit_serializes_independent_sessions() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].max_sessions = 1;
    let report = workflow::run(plan, &root.join("repo"), &root.join("run"), 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval");
    for pair in report.tasks.windows(2) {
        assert!(pair[0].finished_ms.unwrap() <= pair[1].started_ms.unwrap());
    }
    let sessions: std::collections::HashSet<_> = report
        .tasks
        .iter()
        .map(|t| t.session_id.clone().unwrap())
        .collect();
    assert_eq!(sessions.len(), 3);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn repeated_expired_resets_are_delayed_and_automatic_retries_are_bounded() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].args = vec!["quota-known".into()];
    let dir = root.join("run");
    let first = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert_eq!(
        first.tasks[0].restriction.as_ref().unwrap().available_at,
        Some(200)
    );
    assert!(first.tasks[0].retry_not_before > 200);
    let early = workflow::resume_due(&dir, 200, 10).await.unwrap();
    assert_eq!(early.tasks[0].attempt, 1);
    for _ in 0..5 {
        workflow::resume_due(&dir, i64::MAX, 10).await.unwrap();
    }
    let final_report = workflow::load(&dir).unwrap();
    assert_eq!(final_report.tasks[0].attempt, 4);
    workflow::schedule(&dir, "one.txt", "1970-01-01T00:00:01Z").unwrap();
    let manual = workflow::resume_due(&dir, i64::MAX, 10).await.unwrap();
    assert_eq!(manual.tasks[0].attempt, 5);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn quota_is_persisted_manual_schedule_survives_restart_and_due_job_resumes() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].args = vec!["quota-once".into()];
    plan.agents[0]
        .environment
        .insert("MOCK_MARKERS".into(), root.to_str().unwrap().into());
    let dir = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_availability");
    assert!(
        report
            .tasks
            .iter()
            .all(|t| t.status == "waiting_availability")
    );
    workflow::schedule(&dir, "one.txt", "2099-01-01T00:00:00Z").unwrap();
    let report = workflow::resume_due(&dir, 200, 10).await.unwrap();
    assert_eq!(report.tasks[0].status, "waiting_availability");
    workflow::schedule(&dir, "one.txt", "1970-01-01T00:00:01Z").unwrap();
    workflow::schedule(&dir, "two.txt", "1970-01-01T00:00:01Z").unwrap();
    workflow::schedule(&dir, "three.txt", "1970-01-01T00:00:01Z").unwrap();
    let report = workflow::resume_due(&dir, i64::MAX, 10).await.unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert!(report.tasks.iter().all(|t| t.attempt == 2));
    let again = workflow::resume_due(&dir, i64::MAX, 10).await.unwrap();
    assert!(again.tasks.iter().all(|t| t.attempt == 2));
    assert!(dir.join("task-0/process/stdout.jsonl").exists());
    assert!(
        report.tasks[0]
            .artifacts
            .join("process/stdout.jsonl")
            .exists()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn unavailable_role_can_move_to_another_agent_without_foreign_session_id() {
    let (root, mut plan) = support::setup("worktree");
    let mut replacement = plan.agents[0].clone();
    replacement.id = "replacement".into();
    plan.agents[0].args = vec!["quota".into()];
    plan.agents.push(replacement);
    let dir = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_availability");
    for task in ["one.txt", "two.txt", "three.txt"] {
        workflow::reassign(&dir, task, "replacement", None).unwrap();
    }
    assert!(
        workflow::load(&dir)
            .unwrap()
            .tasks
            .iter()
            .all(|t| t.session_id.is_none())
    );
    let report = workflow::resume_due(&dir, i64::MAX, 10).await.unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert!(report.plan.tasks.iter().all(|t| t.agent == "replacement"));
    assert!(dir.join("handoff-0-1.md").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn per_session_models_validate_catalog_and_edits_invalidate_approval() {
    let (root, mut plan) = support::setup("shared");
    let profile = serde_json::to_value(&plan.agents[0]).unwrap();
    let mut value = profile;
    value["catalog"] = serde_json::json!({"models":["local/one","local/two"],"source":"operator_cli_query","queried_at":123,"cli_version":"fixture"});
    value["args"] = serde_json::json!(["task", "{model}", "{session_id}"]);
    plan.agents[0] = serde_json::from_value(value).unwrap();
    let dir = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    workflow::approve(&dir, &report.candidate.as_ref().unwrap().commit).unwrap();
    assert!(workflow::set_model(&dir, "one.txt", "invented/model").is_err());
    workflow::set_model(&dir, "one.txt", "local/two").unwrap();
    assert_eq!(
        workflow::load(&dir).unwrap().plan.tasks[0].model.as_deref(),
        Some("local/two")
    );
    assert!(workflow::merge(&dir).is_err());
    let result = workflow::resume_due(&dir, i64::MAX, 10).await.unwrap();
    assert_eq!(result.state, "waiting_approval");
    assert_eq!(result.tasks[0].attempt, 2);
    assert!(availability::parse_time("bad").is_err());
    std::fs::remove_dir_all(root).unwrap();
}
