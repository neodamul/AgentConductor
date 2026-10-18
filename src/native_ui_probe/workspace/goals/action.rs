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
