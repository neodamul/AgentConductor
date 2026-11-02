#![cfg(unix)]
//! GitHub #331: an update that cannot start says which work blocks it and for how long, tells running work from a failed check,
//! and a run whose coordinator is gone no longer blocks it.
use agentconductor::ui::update_guard::{Blocker, message, run_blocker};
#[path = "support/workflow.rs"]
mod support;

#[test]
fn the_message_names_workspace_work_time_and_the_way_out() {
    let text = message(
        "en",
        &[
            Blocker::Active {
                workspace: "Smart logistics".into(),
                what: "usage query (claude)".into(),
                elapsed_ms: Some(125_000),
            },
            Blocker::Active {
                workspace: "Status endpoint".into(),
                what: "run".into(),
                elapsed_ms: None,
            },
        ],
    );
    assert!(
        text.contains("Smart logistics")
            && text.contains("usage query (claude)")
            && text.contains("2:05"),
        "{text}"
    );
    assert!(
        text.contains("Status endpoint") && !text.contains("None"),
        "{text}"
    );
    assert!(
        text.contains("wait") && text.contains("stop"),
        "the way out: {text}"
    );
}

#[test]
fn a_failed_check_is_not_reported_as_running_work() {
    let text = message(
        "en",
        &[Blocker::Unknown {
            what: "startup job coordinator-ui-1".into(),
            error: "owner.json: permission denied".into(),
        }],
    );
    assert!(
        text.contains("coordinator-ui-1") && text.contains("permission denied"),
        "{text}"
    );
    assert!(
        text.contains("not running work") || text.contains("could not"),
        "{text}"
    );
    assert!(!text.contains("Still running"), "{text}");
}

#[tokio::test]
async fn a_run_marked_running_blocks_only_while_its_coordinator_holds_the_lock() {
    let (root, plan) = support::setup("shared");
    let run = root.join("desktop").join("run");
    std::fs::create_dir_all(root.join("desktop")).unwrap();
    agentconductor::workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    assert!(
        run_blocker("en", "w", &run).is_none(),
        "a finished run does not block"
    );
    let mut store = agentconductor::state::Store::open(run.join("state.sqlite")).unwrap();
    let mut report = store.document("workflow").unwrap();
    report["state"] = "running".into();
    store
        .record("workflow", &report, "test: a run left marked running")
        .unwrap();
    drop(store);
    assert!(
        run_blocker("en", "w", &run).is_none(),
        "state running but no coordinator: the run ended without saying so"
    );
    let _held = agentconductor::leases::Lease::acquire(&run.join("coordinator.lock")).unwrap();
    match run_blocker("en", "w", &run) {
        Some(Blocker::Active { workspace, .. }) => assert_eq!(workspace, "w"),
        other => panic!("a live coordinator must block: {:?}", other.is_some()),
    }
}
