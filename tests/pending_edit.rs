//! GitHub #103: a task that has not started can be reassigned while other tasks run.
#[path = "support/workflow.rs"]
mod support;

use agentconductor::workflow;
use std::time::Duration;

type Run = tokio::task::JoinHandle<Result<workflow::Report, String>>;

async fn wait_until_running(dir: &std::path::Path, run: &mut Run) {
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            if workflow::load(dir).is_ok_and(|r| r.tasks[0].status == "running") {
                return;
            }
            assert!(!run.is_finished(), "the run ended before a task was active");
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("task did not become active within 20 s");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_pending_task_is_reassigned_while_another_task_runs() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].args = vec!["task-slow".into()];
    let mut alt = plan.agents[0].clone();
    alt.id = "alt".into();
    plan.agents.push(alt);
    plan.tasks[2].depends_on = vec![plan.tasks[0].id.clone()];
    let repo = root.join("repo");
    let dir = root.join("run");
    let (run_dir, repo_for_run) = (dir.clone(), repo.clone());
    let mut run =
        tokio::spawn(async move { workflow::run(plan, &repo_for_run, &run_dir, 10).await });
    wait_until_running(&dir, &mut run).await;
    assert_eq!(workflow::load(&dir).unwrap().tasks[2].status, "pending");

    workflow::reassign(&dir, "three.txt", "alt", None).unwrap();
    assert!(
        workflow::reassign(&dir, "one.txt", "alt", None).is_err(),
        "a task that is already running cannot be reassigned"
    );
    assert!(workflow::reassign(&dir, "three.txt", "missing", None).is_err());

    let report = run.await.unwrap().unwrap();
    assert_eq!(report.plan.tasks[2].agent, "alt");
    assert_eq!(report.tasks[2].status, "completed");
    assert_eq!(
        report.plan.tasks[0].agent, "mock",
        "other tasks keep their agent"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_pending_task_model_is_changed_before_it_starts() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].args = vec!["task-slow".into(), "{model}".into()];
    plan.agents[0].catalog = Some(agentconductor::plan::ModelCatalog {
        models: vec!["model-b".into()],
        source: "fixture".into(),
        queried_at: 1,
        cli_version: "test".into(),
        origin: Default::default(),
    });
    plan.tasks[2].depends_on = vec![plan.tasks[0].id.clone()];
    let repo = root.join("repo");
    let dir = root.join("run");
    let (run_dir, repo_for_run) = (dir.clone(), repo.clone());
    let mut run =
        tokio::spawn(async move { workflow::run(plan, &repo_for_run, &run_dir, 10).await });
    wait_until_running(&dir, &mut run).await;

    workflow::set_model(&dir, "three.txt", "model-b").unwrap();
    assert!(workflow::set_model(&dir, "three.txt", "unknown").is_err());
    let report = run.await.unwrap().unwrap();
    assert_eq!(report.plan.tasks[2].model.as_deref(), Some("model-b"));
    assert_eq!(report.tasks[2].status, "completed");
    std::fs::remove_dir_all(root).unwrap();
}
