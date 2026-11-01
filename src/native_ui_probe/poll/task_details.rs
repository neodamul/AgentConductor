use super::super::{Pending, Probe};
use serde_json::Value;

pub(in crate::native_ui_probe) fn apply(probe: &mut Probe, snapshot: Value) {
    let background = matches!(probe.pending, Some(Pending::BackgroundTaskDetails));
    let error = if background {
        None
    } else {
        Some("Task activity response did not match the selection")
    };
    if let Some(view) = probe.room_view.as_mut() {
        if snapshot["task"]["id"].as_str() == view.details_task.as_deref() {
            view.task_details = Some(snapshot);
            probe.pending = None;
            probe.error = None;
        } else {
            probe.pending = None;
            if error.is_some() {
                probe.error = error.map(str::to_owned);
            }
        }
    } else {
        probe.pending = None;
        if !background {
            probe.error = Some("Agent Room is unavailable".into());
        }
    }
}
