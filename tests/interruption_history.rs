//! R-120 / DD-129 (GitHub #385): the run screen states how often the run was interrupted and
//! recovered. Each recovery is listed once, with its latest outcome, oldest first.
use agentconductor::{state::Store, workflow};
use serde_json::json;

#[test]
fn each_recovery_is_listed_once_with_its_latest_outcome() {
    let dir = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    std::fs::create_dir(&dir).unwrap();
    let mut store = Store::open(dir.join("state.sqlite")).unwrap();
    let record = |store: &mut Store, value: serde_json::Value| {
        store.record("auto_recovery", &value, "test").unwrap();
    };
    assert!(workflow::interruptions(&store).unwrap().is_empty());
    record(
        &mut store,
        json!({"status":"resume_pending","recovered":true,"observed_at_ms":1000}),
    );
    record(
        &mut store,
        json!({"status":"resumed","recovered":true,"observed_at_ms":1000}),
    );
    // A stopped run checked again without a lost coordinator is not an interruption.
    record(
        &mut store,
        json!({"status":"recovered","recovered":false,"observed_at_ms":1500}),
    );
    record(
        &mut store,
        json!({"status":"retry_limit","recovered":true,"observed_at_ms":2000}),
    );
    assert_eq!(
        workflow::interruptions(&store).unwrap(),
        [
            json!({"at":1000,"status":"resumed"}),
            json!({"at":2000,"status":"retry_limit"})
        ]
    );
    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[path = "support/workflow.rs"]
mod support;

#[cfg(unix)]
#[tokio::test]
async fn the_run_snapshot_carries_the_interruptions() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    Store::open(run.join("state.sqlite"))
        .unwrap()
        .record(
            "auto_recovery",
            &json!({"status":"resumed","recovered":true,"observed_at_ms":1000}),
            "test",
        )
        .unwrap();
    let room = agentconductor::ui::Room::open(
        &run,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(
        snapshot["interruptions"],
        json!([{"at":1000,"status":"resumed"}])
    );
    drop(room);
    let _ = std::fs::remove_dir_all(root);
}
