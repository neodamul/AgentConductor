#![cfg(unix)]
use agentconductor::{briefing, state::Store, workflow};
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn briefing_covers_all_sessions_and_is_read_only() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    let before = workflow::revision(&report).unwrap();
    let value = briefing::collect(&run, 2000000000000).unwrap();
    assert_eq!(value["sessions"].as_array().unwrap().len(), 3);
    assert_eq!(value["goal"], report.plan.goal);
    assert_eq!(value["sessions"][0]["process"]["state"], "absent");
    assert_eq!(value["approvals"][0]["kind"], "merge");
    assert_eq!(
        workflow::revision(&workflow::load(&run).unwrap()).unwrap(),
        before
    );
    let store = Store::read_only_snapshot(run.join("state.sqlite")).unwrap();
    assert!(store.create("forbidden-query-write").is_err());
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn stale_memory_report_cannot_hide_an_interrupted_process() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    let mut report = workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    report.state = "running".into();
    report.tasks[0].status = "running".into();
    let mut store = Store::open(run.join("state.sqlite")).unwrap();
    store
        .record(
            "workflow",
            &serde_json::to_value(&report).unwrap(),
            "Interrupted fixture",
        )
        .unwrap();
    let session = &report.tasks[0];
    store.record(&format!("cooperative_report/{}/{}",session.id,session.attempt),&serde_json::json!({"agent":"mock","task":session.id,"attempt":session.attempt,"session_id":session.session_id,"reported_at_ms":1000,"phase":"implementation","summary":"Working on authentication","next_step":"Add tests","source":"agentmemory"}),"Fixture memory report").unwrap();
    drop(store);
    let value = briefing::collect(&run, 400000).unwrap();
    assert_eq!(value["sessions"][0]["state"], "running");
    assert_eq!(value["sessions"][0]["process"]["state"], "absent");
    assert_eq!(value["sessions"][0]["memory_report"]["freshness"], "stale");
    assert!(
        value["sessions"][0]["attention"]
            .as_str()
            .unwrap()
            .contains("absent")
    );
    assert_eq!(
        value["sessions"][0]["next_action"],
        "Inspect ownership and recover explicitly"
    );
    assert_eq!(workflow::load(&run).unwrap().tasks[0].status, "running");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_status_query_never_creates_a_missing_run_database() {
    let root = std::env::temp_dir().join(format!(
        "ac-briefing-missing-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    assert!(briefing::collect(&root, 1000).is_err());
    assert!(!root.join("state.sqlite").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn native_and_cli_queries_share_the_same_read_only_contract() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    let before = workflow::revision(&report).unwrap();
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = agentconductor::ui::Room::open(&run, exe).unwrap();
    let packet = room.dispatch(br#"{"kind":"briefing"}"#).unwrap();
    assert_eq!(packet["sessions"].as_array().unwrap().len(), 3);
    assert!(
        room.dispatch(br#"{"kind":"briefing","command":"resume"}"#)
            .is_err()
    );
    let output = std::process::Command::new(exe)
        .arg("briefing")
        .arg(&run)
        .output()
        .unwrap();
    assert!(output.status.success());
    let cli: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(cli["revision"], packet["revision"]);
    assert_eq!(cli["sessions"][0]["task"], packet["sessions"][0]["task"]);
    assert_eq!(
        workflow::revision(&workflow::load(&run).unwrap()).unwrap(),
        before
    );
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn snapshot_stays_consistent_while_a_writer_updates_progress() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    let reader = Store::read_only_snapshot(run.join("state.sqlite")).unwrap();
    let original = reader.document("workflow").unwrap();
    let mut writer = Store::open(run.join("state.sqlite")).unwrap();
    let mut changed = original.clone();
    changed["state"] = serde_json::json!("stopped");
    writer
        .record("workflow", &changed, "Concurrent fixture writer")
        .unwrap();
    assert_eq!(reader.document("workflow").unwrap(), original);
    drop(reader);
    drop(writer);
    assert_eq!(briefing::collect(&run, 1000).unwrap()["state"], "stopped");
    assert_eq!(report.state, "waiting_approval");
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn foreign_session_and_future_reports_do_not_become_current_progress() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    let task = &report.tasks[0];
    let mut store = Store::open(run.join("state.sqlite")).unwrap();
    let key = format!("cooperative_report/{}/{}", task.id, task.attempt);
    let mut value = serde_json::json!({"agent":"mock","task":task.id,"attempt":task.attempt,"session_id":"foreign-session","reported_at_ms":2000,"phase":"implementation","summary":"Still working","next_step":"tests","source":"agentmemory","private_environment":"must not appear"});
    store.record(&key, &value, "Fixture report").unwrap();
    assert_eq!(
        briefing::collect(&run, 1000).unwrap()["sessions"][0]["memory_report"]["freshness"],
        "invalid"
    );
    value["session_id"] = serde_json::json!(task.session_id);
    store.record(&key, &value, "Fixture report").unwrap();
    let b = briefing::collect(&run, 1000).unwrap();
    assert_eq!(b["sessions"][0]["memory_report"]["freshness"], "future");
    assert!(!b.to_string().contains("must not appear"));
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}
