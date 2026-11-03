#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::workflow;

const REQUEST: &str = "Please reconcile the verification result with the run status.";

fn coordinator_file(plan: &agentconductor::plan::Plan, dir: &std::path::Path, mode: &str) {
    let mut coordinator = plan.agents[0].clone();
    coordinator.id = "coordinator".into();
    coordinator.permission_mode = "read_only".into();
    coordinator.executable = support::amend_fixture().to_string_lossy().into_owned();
    coordinator.args = vec![mode.into()];
    std::fs::write(
        agentconductor::native_guard::orchestrator_path(dir),
        serde_json::to_vec(&coordinator).unwrap(),
    )
    .unwrap();
}

async fn failed_run(mode: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let (root, mut plan) = support::setup("shared");
    let root = root.canonicalize().unwrap();
    plan.tasks[0].prompt = "fail".into();
    let run = root.join("run");
    coordinator_file(&plan, &run, mode);
    let report = workflow::run(plan, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    assert_eq!(report.state, "failed");
    (root, run)
}

#[tokio::test]
async fn coordinator_request_is_audited_and_does_not_restart_finished_work() {
    let (root, run) = failed_run("request-answer").await;
    let before = workflow::load(&run).unwrap();
    let proposal = workflow::coordinator_request(&run, 10, REQUEST)
        .await
        .unwrap();
    assert_eq!(proposal.summary, "Reconciled the run");
    let after = workflow::load(&run).unwrap();
    assert_eq!(after.state, "failed");
    assert_eq!(
        after
            .tasks
            .iter()
            .map(|task| task.attempt)
            .collect::<Vec<_>>(),
        before
            .tasks
            .iter()
            .map(|task| task.attempt)
            .collect::<Vec<_>>()
    );
    let rows = workflow::decisions::list(&run).unwrap();
    let requests: Vec<_> = rows.iter().filter(|row| row["kind"] == "request").collect();
    assert_eq!(
        requests
            .iter()
            .map(|row| row["judgment"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["queued", "processing", "completed"]
    );
    assert!(
        requests.last().unwrap()["detail"]
            .as_str()
            .unwrap()
            .contains("Reconciled the run")
    );
    assert!(workflow::merge(&run).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn coordinator_failure_is_visible_without_changing_task_attempts() {
    let (root, run) = failed_run("request-fail").await;
    let before = workflow::load(&run).unwrap();
    assert!(
        workflow::coordinator_request(&run, 10, REQUEST)
            .await
            .is_err()
    );
    let after = workflow::load(&run).unwrap();
    assert_eq!(after.state, "failed");
    assert_eq!(
        after
            .tasks
            .iter()
            .map(|task| task.attempt)
            .collect::<Vec<_>>(),
        before
            .tasks
            .iter()
            .map(|task| task.attempt)
            .collect::<Vec<_>>()
    );
    let requests: Vec<_> = workflow::decisions::list(&run)
        .unwrap()
        .into_iter()
        .filter(|row| row["kind"] == "request")
        .collect();
    assert_eq!(requests.last().unwrap()["judgment"], "failed");
    assert!(
        !requests.last().unwrap()["detail"]
            .as_str()
            .unwrap()
            .is_empty()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn coordinator_request_waits_for_the_run_lease_and_records_timeout() {
    let (root, run) = failed_run("request-answer").await;
    let lease = agentconductor::leases::Lease::acquire(&run.join("coordinator.lock")).unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(650));
        drop(lease);
    });
    workflow::coordinator_request(&run, 5, REQUEST)
        .await
        .unwrap();
    release.join().unwrap();
    let states: Vec<_> = workflow::decisions::list(&run)
        .unwrap()
        .into_iter()
        .filter(|row| row["kind"] == "request")
        .map(|row| row["judgment"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(states, ["queued", "processing", "completed"]);
    std::fs::remove_dir_all(root).unwrap();

    let (root, run) = failed_run("request-answer").await;
    let _lease = agentconductor::leases::Lease::acquire(&run.join("coordinator.lock")).unwrap();
    assert!(
        workflow::coordinator_request(&run, 1, REQUEST)
            .await
            .is_err()
    );
    let states: Vec<_> = workflow::decisions::list(&run)
        .unwrap()
        .into_iter()
        .filter(|row| row["kind"] == "request")
        .map(|row| row["judgment"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(states, ["queued", "failed"]);
    drop(_lease);
    std::fs::remove_dir_all(root).unwrap();
}
