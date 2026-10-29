use serde_json::Value;

use super::super::{Pending, Probe};

impl Probe {
    pub(super) fn refresh_after(&mut self, pending: Pending) {
        let result = self
            .worker
            .as_ref()
            .unwrap()
            .send(br#"{"kind":"snapshot","icons":true}"#);
        if let Err(error) = result {
            self.pending = None;
            self.error = Some(error);
        } else {
            self.pending = Some(pending);
        }
    }
}

pub(super) fn load_workspace(probe: &mut Probe, snapshot: &Value) {
    super::super::workspace::chat::load_history(probe, snapshot);
    probe.goal_optimizer.restore(snapshot);
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

pub(super) fn navigation_refresh(probe: &mut Probe, snapshot: Value) {
    load_workspace(probe, &snapshot);
    probe.snapshot = Some(snapshot);
    probe.pending = None;
    probe.error = None;
    probe.navigate(super::super::workspace::navigation::Event::List);
}

pub(super) fn stop_plan_reply(probe: &mut Probe, snapshot: &Value) {
    if matches!(probe.pending.as_ref(), Some(Pending::StopPlanningRefresh)) {
        stop_plan_refresh(probe, snapshot);
    } else if snapshot["stopping"] == true
        || probe.snapshot.as_ref().is_none_or(|current| {
            current["job"]["state"] != "running" || current["job"]["kind"] != "plan"
        })
    {
        probe.refresh_workspace();
    } else {
        probe.navigation_failed();
        probe.pending = None;
        probe.error = Some("Planning cancellation was not acknowledged".into());
    }
}

pub(super) fn stop_plan_refresh(probe: &mut Probe, snapshot: &Value) {
    let Some(super::super::workspace::navigation::Event::Delete { path, .. }) =
        probe.navigation.delete_after_stop()
    else {
        probe.navigation_failed();
        probe.pending = None;
        probe.error = Some("Confirmed planning deletion state was lost".into());
        return;
    };
    if snapshot["root"].as_str() != Some(&path) {
        probe.navigation_failed();
        probe.pending = None;
        probe.error = Some("Workspace changed while planning cancellation was pending".into());
        return;
    }
    load_workspace(probe, snapshot);
    probe.snapshot = Some(snapshot.clone());
    probe.pending = None;
    if super::super::workspace::navigation::stop::can_archive_after_stop(snapshot) {
        let event = probe.navigation.delete_after_stop().unwrap();
        probe.navigation.finish_stop_for_delete();
        probe.navigate(event);
    } else {
        probe.workspace_next_refresh = None;
    }
}
