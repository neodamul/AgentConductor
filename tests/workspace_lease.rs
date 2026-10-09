#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::workflow;

#[tokio::test]
async fn rejected_dirty_repository_does_not_leave_an_unknown_run_claim() {
    let (root, plan) = support::setup("shared");
    let repo = root.join("repo");
    let dirty = repo.join("unexpected.txt");
    std::fs::write(&dirty, "existing user work").unwrap();
    assert!(
        workflow::run(plan.clone(), &repo, &root.join("rejected"), 10)
            .await
            .is_err()
    );
    std::fs::remove_file(dirty).unwrap();
    let report = workflow::run(plan, &repo, &root.join("valid"), 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval");
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn independent_coordinators_cannot_mutate_the_same_repository() {
    let (root, mut first) = support::setup("shared");
    let second = first.clone();
    first.agents[0].args = vec!["task-slow".into()];
    let repo = root.join("repo");
    let dir = root.join("run");
    let workrepo = repo.clone();
    let workdir = dir.clone();
    let job = tokio::spawn(async move { workflow::run(first, &workrepo, &workdir, 10).await });
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while !std::fs::metadata(dir.join("task-0/process/stdout.jsonl")).is_ok_and(|m| m.len() > 0)
        {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let concurrent = workflow::run(second, &repo, &root.join("second"), 10).await;
    workflow::stop(&dir, "test cleanup").unwrap();
    let stopped = job.await.unwrap().unwrap();
    assert_eq!(stopped.state, "stopped");
    assert!(
        concurrent.is_err(),
        "second coordinator must fail before spawning agents"
    );
    assert!(!root.join("second").exists());
    std::fs::remove_dir_all(root).unwrap();
}
