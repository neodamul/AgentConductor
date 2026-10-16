#![cfg(unix)]
use agentconductor::{
    cooperative::{self, AckState, Acknowledgment, Address},
    state::Store,
    workflow,
};
#[path = "support/cooperative.rs"]
mod cooperative_support;
#[path = "support/workflow.rs"]
mod support;
use cooperative_support::{issue, setup};
fn ack(record: &cooperative::Directive, state: AckState) -> Acknowledgment {
    Acknowledgment {
        id: record.id.clone(),
        address: record.address.clone(),
        state,
        reported_at_ms: 1500,
        reason: "Checkpoint acknowledged".into(),
    }
}
#[test]
fn addressed_lifecycle_is_durable_idempotent_and_audited() {
    let (root, mut store, report) = setup();
    let original = workflow::revision(&report).unwrap();
    let record = cooperative::issue(&mut store, issue(&report), 1000).unwrap();
    assert_eq!(
        cooperative::poll(&mut store, &record.address, 1200)
            .unwrap()
            .len(),
        1
    );
    let accepted = ack(&record, AckState::Accepted);
    cooperative::acknowledge(&mut store, accepted.clone(), 1500).unwrap();
    cooperative::acknowledge(&mut store, accepted, 1500).unwrap();
    assert_eq!(
        cooperative::poll(&mut store, &record.address, 4000)
            .unwrap()
            .len(),
        1
    );
    let mut done = ack(&record, AckState::Completed);
    done.reported_at_ms = 4000;
    cooperative::acknowledge(&mut store, done, 4000).unwrap();
    assert!(
        cooperative::poll(&mut store, &record.address, 4001)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        workflow::revision(&serde_json::from_value(store.document("workflow").unwrap()).unwrap())
            .unwrap(),
        original
    );
    assert!(
        store
            .pending_memory()
            .unwrap()
            .iter()
            .any(|(_, v)| v.contains(&record.id))
    );
    drop(store);
    let mut reopened = Store::open(root.join("state.sqlite")).unwrap();
    assert!(
        cooperative::poll(&mut reopened, &record.address, 5000)
            .unwrap()
            .is_empty()
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn wrong_session_stale_plan_and_conflicting_acknowledgments_fail() {
    let (root, mut store, report) = setup();
    let record = cooperative::issue(&mut store, issue(&report), 1000).unwrap();
    let mut wrong = ack(&record, AckState::Accepted);
    wrong.address.session_id = "foreign".into();
    assert!(cooperative::acknowledge(&mut store, wrong, 1500).is_err());
    assert!(cooperative::issue(&mut store, issue(&report), 1200).is_err());
    assert!(cooperative::acknowledge(&mut store, ack(&record, AckState::Completed), 1500).is_err());
    let accepted = ack(&record, AckState::Accepted);
    cooperative::acknowledge(&mut store, accepted.clone(), 1500).unwrap();
    let mut changed = accepted;
    changed.reason = "Different evidence".into();
    assert!(cooperative::acknowledge(&mut store, changed, 1600).is_err());
    let mut plan_changed = report.clone();
    plan_changed.plan.tasks[0].prompt.push_str(" revised");
    store
        .record(
            "workflow",
            &serde_json::to_value(plan_changed).unwrap(),
            "Fixture plan change",
        )
        .unwrap();
    assert!(cooperative::poll(&mut store, &record.address, 1700).is_err());
    assert!(cooperative::acknowledge(&mut store, ack(&record, AckState::Rejected), 1700).is_err());
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn expiration_unknown_fields_and_stale_issue_are_rejected() {
    let (root, mut store, report) = setup();
    let mut request = issue(&report);
    request.revision = "0".repeat(64);
    assert!(cooperative::issue(&mut store, request, 1000).is_err());
    let record = cooperative::issue(&mut store, issue(&report), 1000).unwrap();
    assert!(
        cooperative::poll(&mut store, &record.address, 3000)
            .unwrap()
            .is_empty()
    );
    assert!(cooperative::acknowledge(&mut store, ack(&record, AckState::Accepted), 3000).is_err());
    let mut address: serde_json::Value = serde_json::to_value(&record.address).unwrap();
    address["command"] = serde_json::json!("deploy");
    assert!(serde_json::from_value::<Address>(address).is_err());
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn intermediate_reports_are_monotonic_and_cannot_change_execution_state() {
    let (root, mut store, report) = setup();
    let record = cooperative::issue(&mut store, issue(&report), 1000).unwrap();
    let progress = cooperative::Progress {
        address: record.address.clone(),
        sequence: 1,
        reported_at_ms: 1500,
        phase: "implementation".into(),
        summary: "Implementing refresh token rotation".into(),
        next_step: "Run edge-case tests".into(),
    };
    assert!(cooperative::report_progress(&mut store, progress.clone(), 1600).unwrap());
    assert!(!cooperative::report_progress(&mut store, progress.clone(), 1600).unwrap());
    let mut conflict = progress.clone();
    conflict.summary = "Changed same sequence".into();
    assert!(cooperative::report_progress(&mut store, conflict, 1600).is_err());
    let mut older = progress.clone();
    older.sequence = 0;
    assert!(cooperative::report_progress(&mut store, older, 1600).is_err());
    let mut future = progress.clone();
    future.sequence = 2;
    future.reported_at_ms = 2000;
    assert!(cooperative::report_progress(&mut store, future, 1600).is_err());
    let stored = store
        .document(&format!("cooperative_report/{}/1", record.address.task))
        .unwrap();
    assert_eq!(stored["summary"], progress.summary);
    assert_eq!(stored["source"], "local checkpoint adapter");
    assert_eq!(
        store.document("workflow").unwrap()["tasks"][0]["status"],
        "running"
    );
    let bundle = agentconductor::briefing::collect(&root, 1600).unwrap();
    assert_eq!(
        bundle["sessions"][0]["memory_report"]["freshness"],
        "recent"
    );
    assert_eq!(
        bundle["sessions"][0]["memory_report"]["summary"],
        progress.summary
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn concurrent_issue_has_one_winner_and_pending_expiry_survives_ack_failure() {
    let (root, mut store, report) = setup();
    let record = cooperative::issue(&mut store, issue(&report), 1000).unwrap();
    assert!(cooperative::acknowledge(&mut store, ack(&record, AckState::Accepted), 3000).is_err());
    assert_eq!(
        store
            .document(&format!("cooperative_directive/{}", record.id))
            .unwrap()["state"],
        "expired"
    );
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let jobs = (0..2)
        .map(|_| {
            let path = root.join("state.sqlite");
            let b = barrier.clone();
            let mut request = issue(&report);
            request.issued_at_ms = 4000;
            request.expires_at_ms = 5000;
            std::thread::spawn(move || {
                let mut writer = Store::open(path).unwrap();
                b.wait();
                cooperative::issue(&mut writer, request, 4000).is_ok()
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        jobs.into_iter()
            .filter_map(|j| j.join().unwrap().then_some(()))
            .count(),
        1
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn identical_workflows_and_reused_sessions_have_separate_run_namespaces() {
    let (root, mut first, report) = setup();
    let other = root.join("other.sqlite");
    let mut second = Store::open(&other).unwrap();
    second
        .record(
            "workflow",
            &serde_json::to_value(&report).unwrap(),
            "Identical workflow",
        )
        .unwrap();
    let a = cooperative::issue(&mut first, issue(&report), 1000).unwrap();
    let b = cooperative::issue(&mut second, issue(&report), 1000).unwrap();
    assert_ne!(a.address.run_id, b.address.run_id);
    assert_ne!(a.id, b.id);
    assert!(cooperative::poll(&mut second, &a.address, 1200).is_err());
    assert!(cooperative::acknowledge(&mut second, ack(&a, AckState::Accepted), 1500).is_err());
    drop(first);
    drop(second);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn changed_plan_supersedes_old_directive_without_reusing_its_acknowledgment() {
    let (root, mut store, mut report) = setup();
    let old = cooperative::issue(&mut store, issue(&report), 1000).unwrap();
    cooperative::acknowledge(&mut store, ack(&old, AckState::Accepted), 1500).unwrap();
    report.plan.tasks[0]
        .prompt
        .push_str(" revised scope-preserving instructions");
    store
        .record(
            "workflow",
            &serde_json::to_value(&report).unwrap(),
            "Reviewed plan revision",
        )
        .unwrap();
    let mut request = issue(&report);
    request.issued_at_ms = 1600;
    let new = cooperative::issue(&mut store, request, 1600).unwrap();
    assert_ne!(new.id, old.id);
    assert_eq!(
        store
            .document(&format!("cooperative_directive/{}", old.id))
            .unwrap()["state"],
        "superseded"
    );
    assert!(cooperative::acknowledge(&mut store, ack(&old, AckState::Completed), 1700).is_err());
    assert_eq!(
        cooperative::poll(&mut store, &new.address, 1700).unwrap()[0].id,
        new.id
    );
    assert_eq!(
        store.document("workflow").unwrap()["tasks"][0]["status"],
        "running"
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[path = "cooperative/memory.rs"]
mod memory;
