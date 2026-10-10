#![cfg(unix)]
use agentconductor::{
    cooperative::{self, AckState, Acknowledgment, Action, Issue},
    state::Store,
    workflow,
};
#[path = "support/workflow.rs"]
mod support;

fn setup(
    state: &str,
    action: Action,
) -> (
    std::path::PathBuf,
    Store,
    workflow::Report,
    cooperative::Directive,
) {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].cooperative = agentconductor::plan::CooperativeMode::CheckpointV1;
    plan.agents[0].args.push("{session_id}".into());
    let mut report: workflow::Report = serde_json::from_value(serde_json::json!({
        "schema_version":1,"state":"running","repo":root.join("repo"),"base":"base","branch":"main","plan":plan,
        "tasks":plan.tasks.iter().map(|t|serde_json::json!({"id":t.id,"status":"running","attempt":1,"cwd":root.join("repo"),"started_ms":100,"finished_ms":null,"session_id":format!("session-{}",t.id),"error":null,"base":"base","snapshot":null,"artifacts":root.join(&t.id)})).collect::<Vec<_>>(),
        "candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":root.join("repo")
    })).unwrap();
    let mut store = Store::open(root.join("state.sqlite")).unwrap();
    store
        .record(
            "workflow",
            &serde_json::to_value(&report).unwrap(),
            "Fixture running",
        )
        .unwrap();
    let directive = cooperative::issue(
        &mut store,
        Issue {
            revision: workflow::revision(&report).unwrap(),
            task: "one.txt".into(),
            action,
            instruction: "Use revised content".into(),
            issued_at_ms: 1000,
            expires_at_ms: 3000,
        },
        1000,
    )
    .unwrap();
    if state != "pending" {
        cooperative::acknowledge(
            &mut store,
            Acknowledgment {
                id: directive.id.clone(),
                address: directive.address.clone(),
                state: AckState::Accepted,
                reported_at_ms: 1500,
                reason: "Accepted before interruption".into(),
            },
            1500,
        )
        .unwrap();
    }
    report.state = "stopped".into();
    report.tasks[0].status = "stopped".into();
    store
        .record(
            "workflow",
            &serde_json::to_value(&report).unwrap(),
            "Fixture stopped after interruption",
        )
        .unwrap();
    (root, store, report, directive)
}

#[test]
fn explicit_resume_preserves_pending_and_accepted_rework_as_idempotent_context() {
    for state in ["pending", "accepted"] {
        let (root, mut store, report, directive) = setup(state, Action::Rework);
        let context = cooperative::prepare_continuation(&mut store, "one.txt", 2000)
            .unwrap()
            .unwrap();
        assert_eq!(context.directive_id, directive.id);
        assert_eq!(context.instruction, "Use revised content");
        assert_eq!(context.source, "local checkpoint ledger");
        assert_eq!(context.address, directive.address);
        let saved = store
            .document(&format!("cooperative_directive/{}", directive.id))
            .unwrap();
        assert_eq!(saved["state"], "superseded");
        assert_eq!(
            cooperative::prepare_continuation(&mut store, "one.txt", 2100)
                .unwrap()
                .unwrap(),
            context
        );
        assert_eq!(
            workflow::revision(
                &serde_json::from_value(store.document("workflow").unwrap()).unwrap()
            )
            .unwrap(),
            workflow::revision(&report).unwrap()
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn expired_pending_and_foreign_bindings_do_not_become_continuation() {
    let (root, mut store, _, _) = setup("pending", Action::Rework);
    assert!(
        cooperative::prepare_continuation(&mut store, "one.txt", 4000)
            .unwrap()
            .is_none()
    );
    std::fs::remove_dir_all(root).unwrap();
    let (root, mut store, mut report, _) = setup("accepted", Action::Rework);
    report.tasks[0].session_id = Some("foreign".into());
    store
        .record(
            "workflow",
            &serde_json::to_value(report).unwrap(),
            "Changed native binding",
        )
        .unwrap();
    assert!(cooperative::prepare_continuation(&mut store, "one.txt", 2000).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn resumed_pause_or_handoff_is_superseded_without_replaying_stop_instructions() {
    for action in [Action::Pause, Action::Handoff] {
        let (root, mut store, _, d) = setup("accepted", action);
        assert!(
            cooperative::prepare_continuation(&mut store, "one.txt", 2000)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store
                .document(&format!("cooperative_directive/{}", d.id))
                .unwrap()["state"],
            "superseded"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn completed_or_rejected_rework_is_not_replayed() {
    for state in [AckState::Completed, AckState::Rejected] {
        let (root, mut store, report, d) = setup("accepted", Action::Rework);
        let mut running = report.clone();
        running.state = "running".into();
        running.tasks[0].status = "running".into();
        store
            .record(
                "workflow",
                &serde_json::to_value(running).unwrap(),
                "Fixture active acknowledgment",
            )
            .unwrap();
        cooperative::acknowledge(
            &mut store,
            Acknowledgment {
                id: d.id.clone(),
                address: d.address,
                state,
                reported_at_ms: 1800,
                reason: "Resolved before stop".into(),
            },
            1800,
        )
        .unwrap();
        store
            .record(
                "workflow",
                &serde_json::to_value(report).unwrap(),
                "Fixture stopped",
            )
            .unwrap();
        assert!(
            cooperative::prepare_continuation(&mut store, "one.txt", 2000)
                .unwrap()
                .is_none()
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
