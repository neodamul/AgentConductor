use super::super::{Pending, Probe};

pub(in crate::native_ui_probe::workspace) fn request_new_goal(probe: &mut Probe) {
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
                .send(&super::action::new_goal_request(revision))
        });
    match result {
        Ok(()) => {
            probe.pending = Some(Pending::NewGoal);
            probe.error = None;
        }
        Err(error) => probe.error = Some(error),
    }
}
