#![cfg(unix)]
use agentconductor::plan::Plan;
#[path = "support/workflow.rs"]
mod support;
#[path = "support/cooperative_worker.rs"]
mod worker;
use worker::{checkpoint_case, checkpoint_case_with_ack, checkpoint_case_with_crash};

#[test]
fn cooperative_capability_requires_owned_custom_session_and_reserved_environment() {
    let (root, plan) = support::setup("shared");
    let mut value = serde_json::to_value(plan).unwrap();
    assert!(
        value["agents"][0].get("cooperative").is_none(),
        "disabled capability must preserve existing plan digests"
    );
    value["agents"][0]["cooperative"] = "checkpoint_v1".into();
    let mut p: Plan = serde_json::from_value(value).expect("explicit capability registration");
    assert!(p.validate().is_err(), "session argv binding required");
    p.agents[0].args.push("{session_id}".into());
    p.validate().unwrap();
    p.agents[0]
        .environment
        .insert("AC_CHECKPOINT_RUN_DIR".into(), "foreign".into());
    assert!(
        p.validate().is_err(),
        "bootstrap environment must be coordinator-owned"
    );
    p.agents[0].environment.clear();
    p.agents[0].parser = agentconductor::protocol::Provider::Codex;
    assert!(
        p.validate().is_err(),
        "provider-specific initial session contract not verified"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn compiled_worker_polls_before_and_during_work_and_reports_without_granting_merge() {
    for mode in ["shared", "worktree"] {
        checkpoint_case(mode, false, "rework");
    }
}

#[test]
fn conflicting_returned_session_cannot_replace_the_owned_checkpoint_identity() {
    checkpoint_case("shared", true, "rework");
}

#[test]
fn acknowledged_pause_and_handoff_cannot_become_completed_work_or_restart_automatically() {
    for action in ["pause", "handoff"] {
        checkpoint_case("shared", false, action);
    }
}

#[test]
fn pending_or_accepted_directive_at_clean_exit_blocks_completion() {
    for ack in ["pending", "accepted"] {
        checkpoint_case_with_ack("shared", false, "rework", ack);
    }
}

#[test]
fn interrupted_coordinator_keeps_checkpoint_ownership_until_worker_exit_then_requires_resume() {
    for action in ["pause", "handoff"] {
        checkpoint_case_with_crash("shared", false, action, "completed", true);
    }
}

#[test]
fn interrupted_accepted_rework_reaches_the_resumed_worker_with_partial_files() {
    checkpoint_case_with_crash("shared", false, "rework", "accepted", true);
}
