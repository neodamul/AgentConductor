use super::super::{Pending, Probe};
use serde_json::json;

pub fn next_phase_request(revision: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({"kind":"next_phase","revision":revision}))
        .expect("next-phase request is serializable")
}

pub fn select_goal_request(revision: &str, slug: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({"kind":"select_goal","revision":revision,"slug":slug}))
        .expect("goal selection request is serializable")
}

pub fn new_goal_request(revision: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({"kind":"new_goal","revision":revision}))
        .expect("new-goal request is serializable")
}

pub fn can_start_new_goal(snapshot: &serde_json::Value, pending: bool, dirty: bool) -> bool {
    let run_state = snapshot["run_state"].as_str();
    !pending
        && !dirty
        && snapshot["job"]["state"] != "running"
        && snapshot["config"]["repository"]
            .as_str()
            .is_some_and(|repository| !repository.trim().is_empty())
        && run_state.is_none_or(|state| matches!(state, "completed" | "failed" | "stopped"))
}

pub(super) fn request_next_phase(probe: &mut Probe) {
    let result = probe
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot["revision"].as_str())
        .ok_or_else(|| "Workspace revision is unavailable".to_owned())
        .and_then(|revision| {
            probe
                .worker
                .as_ref()
                .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                .send(&next_phase_request(revision))
        });
    match result {
        Ok(()) => {
            probe.pending = Some(Pending::NextPhase);
            probe.error = None;
        }
        Err(error) => probe.error = Some(error),
    }
}

pub(super) fn request_select_goal(probe: &mut Probe, slug: &str) {
    let result = probe
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot["revision"].as_str())
        .ok_or_else(|| "Workspace revision is unavailable".to_owned())
        .and_then(|revision| {
            probe
                .worker
                .as_ref()
                .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                .send(&select_goal_request(revision, slug))
        });
    match result {
        Ok(()) => {
            probe.pending = Some(Pending::SelectGoal);
            probe.error = None;
        }
        Err(error) => probe.error = Some(error),
    }
}
