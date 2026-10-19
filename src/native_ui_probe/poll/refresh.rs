use super::super::Probe;
use serde_json::Value;

pub(super) fn apply(probe: &mut Probe, snapshot: Value) {
    if let Some(view) = probe.room_view.as_mut() {
        if view
            .evidence
            .as_ref()
            .is_some_and(|evidence| evidence["revision"] != snapshot["revision"])
        {
            view.evidence = None;
            view.diff.clear();
        }
    } else {
        load_workspace(probe, &snapshot);
    }
    probe.snapshot = Some(snapshot);
    probe.pending = None;
    probe.error = None;
    if probe.resume_after_tool_approval
        && probe
            .snapshot
            .as_ref()
            .is_some_and(|s| s["job"]["kind"] == "tool_approve")
        && probe
            .snapshot
            .as_ref()
            .is_some_and(|s| s["job"]["state"] != "running")
    {
        let resume = probe.snapshot.as_ref().is_some_and(|s| {
            s["job"]["state"] != "failed"
                && matches!(s["report"]["state"].as_str(), Some("ready" | "stopped"))
        });
        probe.resume_after_tool_approval = false;
        if resume {
            probe.resume_room();
        }
    }
}

pub(super) fn load_workspace(probe: &mut Probe, snapshot: &Value) {
    probe.goal = snapshot["config"]["goal"]
        .as_str()
        .unwrap_or_default()
        .into();
    probe.name = snapshot["config"]["name"]
        .as_str()
        .unwrap_or_default()
        .into();
    probe.repository = snapshot["config"]["repository"]
        .as_str()
        .unwrap_or_default()
        .into();
    probe.workspace_mode =
        serde_json::from_value(snapshot["config"]["workspace_mode"].clone()).unwrap_or_default();
    probe.profiles = snapshot["config"]["profiles"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    probe.coordinator = snapshot["config"]["coordinator"]
        .as_str()
        .unwrap_or_default()
        .into();
    probe.roles = serde_json::from_value(snapshot["config"]["roles"].clone()).unwrap_or_default();
}
