use agentconductor::{
    gates::{Ledger, ToolRequest},
    state::Store,
    workflow,
};
use serde_json::json;
use std::{fs, path::Path};

#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn active_waiting_worker_refresh_is_queued_then_committed_by_its_coordinator() {
    let (root, dir, mut report, id, _request) = pending_run().await;
    let task = &report.tasks[0];
    fs::create_dir_all(task.artifacts.join("process")).unwrap();
    fs::write(
        task.artifacts.join("process/owner.json"),
        r#"{"coordinator_pid":1,"pid":42,"process_group":42,"phase":"running","exit_code":null,"stopped":false,"timed_out":false}"#,
    ).unwrap();
    let _coordinator =
        agentconductor::leases::Lease::acquire(&dir.join("coordinator.lock")).unwrap();

    workflow::queue_live_tool_refresh(&dir, &id).unwrap();
    assert!(
        workflow::queue_live_tool_refresh(&dir, &id)
            .unwrap_err()
            .contains("already queued")
    );
    assert!(
        workflow::apply_queued_tool_refresh(
            &dir,
            &mut report,
            &mut Store::open(dir.join("state.sqlite")).unwrap(),
            Path::new(env!("CARGO_BIN_EXE_agentconductor")),
        )
        .await
        .unwrap()
    );

    let current = workflow::load(&dir).unwrap();
    let replacement = current.tasks[0].tool_request.as_deref().unwrap();
    let ledger = Ledger::open(&dir.join("state.sqlite")).unwrap();
    assert_eq!(ledger.receipt(&id).unwrap().unwrap().status, "superseded");
    assert_eq!(
        ledger.receipt(replacement).unwrap().unwrap().status,
        "pending"
    );
    assert_eq!(
        report.tasks[0].approval_epoch,
        current.tasks[0].approval_epoch
    );
    assert_eq!(report.tasks[0].tool_request.as_deref(), Some(replacement));
    assert_eq!(current.tasks[0].approval_epoch, 1);
    drop((ledger, _coordinator));
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn wrong_receipt_is_rejected_and_worker_exit_discards_queued_refresh() {
    let (root, dir, mut report, id, _) = pending_run().await;
    assert!(workflow::queue_live_tool_refresh(&dir, "wrong-receipt").is_err());
    set_active_process(&report.tasks[0].artifacts);
    workflow::queue_live_tool_refresh(&dir, &id).unwrap();
    fs::write(dir.join("tool-refresh-partial.pending"), b"{").unwrap();
    let owner_path = report.tasks[0].artifacts.join("process/owner.json");
    let mut owner: serde_json::Value =
        serde_json::from_slice(&fs::read(&owner_path).unwrap()).unwrap();
    owner["phase"] = "terminal".into();
    fs::write(&owner_path, serde_json::to_vec(&owner).unwrap()).unwrap();
    let mut store = Store::open(dir.join("state.sqlite")).unwrap();

    assert!(
        !workflow::apply_queued_tool_refresh(
            &dir,
            &mut report,
            &mut store,
            Path::new(env!("CARGO_BIN_EXE_agentconductor")),
        )
        .await
        .unwrap()
    );
    assert_eq!(report.tasks[0].approval_epoch, 0);
    assert_eq!(
        Ledger::open(&dir.join("state.sqlite"))
            .unwrap()
            .receipt(&id)
            .unwrap()
            .unwrap()
            .status,
        "pending"
    );
    assert!(dir.join("tool-refresh-partial.pending").exists());
    fs::remove_file(dir.join("tool-refresh-partial.pending")).unwrap();
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn raced_denial_rolls_back_epoch_and_discards_refresh_without_aborting_run() {
    let (root, dir, mut report, id, _) = pending_run().await;
    set_active_process(&report.tasks[0].artifacts);
    workflow::queue_live_tool_refresh(&dir, &id).unwrap();
    let original = Ledger::open(&dir.join("state.sqlite"))
        .unwrap()
        .receipt(&id)
        .unwrap()
        .unwrap();
    let mut store = Store::open(dir.join("state.sqlite")).unwrap();
    let engine = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let mut applying = Box::pin(workflow::apply_queued_tool_refresh(
        &dir,
        &mut report,
        &mut store,
        engine,
    ));
    let mut denial_injected = false;
    let result = loop {
        tokio::select! {
            outcome = &mut applying => break outcome.unwrap(),
            _ = tokio::time::sleep(std::time::Duration::from_millis(10)), if !denial_injected => {
                if workflow::load(&dir).unwrap().tasks[0].approval_epoch == 1 {
                    Ledger::open(&dir.join("state.sqlite")).unwrap()
                        .decide(&id, &id, false, &original.context, 200, 200).unwrap();
                    denial_injected = true;
                }
            }
        }
    };

    drop(applying);
    assert!(denial_injected);
    assert!(!result);
    assert_eq!(workflow::load(&dir).unwrap().tasks[0].approval_epoch, 0);
    assert_eq!(report.tasks[0].approval_epoch, 0);
    assert_eq!(
        Ledger::open(&dir.join("state.sqlite"))
            .unwrap()
            .receipt(&id)
            .unwrap()
            .unwrap()
            .status,
        "denied"
    );
    assert!(!dir.join("tool-refresh.request").exists());
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

async fn pending_run() -> (
    std::path::PathBuf,
    std::path::PathBuf,
    agentconductor::workflow::Report,
    String,
    ToolRequest,
) {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    let mut report = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    report.state = "running".into();
    report.tasks[0].status = "running".into();
    report.tasks[0].session_id = Some("test-session".into());
    persist(&dir, &report);
    let request = ToolRequest {
        tool: "Bash".into(),
        input: json!({"command":"curl -sS https://example.invalid"}),
        reason: "External command requires human approval".into(),
    };
    let context = workflow::tool_context(&dir, "one.txt", &request).unwrap();
    let id = Ledger::open(&dir.join("state.sqlite"))
        .unwrap()
        .request(&context, &request, 100)
        .unwrap();
    report.tasks[0].tool_request = Some(id.clone());
    persist(&dir, &report);
    (root, dir, report, id, request)
}

fn persist(dir: &Path, report: &agentconductor::workflow::Report) {
    let value = serde_json::to_value(report).unwrap();
    Store::open(dir.join("state.sqlite"))
        .unwrap()
        .record("workflow", &value, "test active run")
        .unwrap();
    fs::write(
        dir.join("report.json"),
        serde_json::to_vec_pretty(report).unwrap(),
    )
    .unwrap();
}

fn set_active_process(artifacts: &Path) {
    fs::create_dir_all(artifacts.join("process")).unwrap();
    fs::write(
        artifacts.join("process/owner.json"),
        r#"{"coordinator_pid":1,"pid":42,"process_group":42,"phase":"running","exit_code":null,"stopped":false,"timed_out":false}"#,
    ).unwrap();
}
