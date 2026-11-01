use serde::Serialize;
use serde_json::Value;

use super::super::{Pending, Probe};

#[cfg(test)]
#[path = "../../../tests/native_ui_probe/active_work_poll.rs"]
mod active_work_poll_tests;

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
    load_workspace_fields(probe, snapshot, true);
}

pub(super) fn load_background_workspace(probe: &mut Probe, snapshot: &Value) {
    let dirty_name = field_is_dirty(probe, "name", &probe.name);
    let dirty_goal = field_is_dirty(probe, "goal", &probe.goal);
    let dirty_repository = field_is_dirty(probe, "repository", &probe.repository);
    let dirty_mode = field_is_dirty(probe, "workspace_mode", &probe.workspace_mode);
    let dirty_profiles = field_is_dirty(probe, "profiles", &probe.profiles);
    let dirty_coordinator = field_is_dirty(probe, "coordinator", &probe.coordinator);
    let dirty_roles = field_is_dirty(probe, "roles", &probe.roles);
    let (name, goal, repository, mode, profiles, coordinator, roles) = (
        probe.name.clone(),
        probe.goal.clone(),
        probe.repository.clone(),
        probe.workspace_mode,
        probe.profiles.clone(),
        probe.coordinator.clone(),
        probe.roles.clone(),
    );

    load_workspace_fields(probe, snapshot, !dirty_goal);
    if dirty_name {
        probe.name = name;
    }
    if dirty_goal {
        probe.goal = goal;
    }
    if dirty_repository {
        probe.repository = repository;
    }
    if dirty_mode {
        probe.workspace_mode = mode;
    }
    if dirty_profiles {
        probe.profiles = profiles;
    }
    if dirty_coordinator {
        probe.coordinator = coordinator;
    }
    if dirty_roles {
        probe.roles = roles;
    }
}

fn field_is_dirty<T: Serialize>(probe: &Probe, field: &str, draft: &T) -> bool {
    let Some(previous) = probe
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot["config"].get(field))
    else {
        return false;
    };
    serde_json::to_value(draft).is_ok_and(|draft| draft != *previous)
}

fn load_workspace_fields(probe: &mut Probe, snapshot: &Value, restore_optimizer: bool) {
    super::super::workspace::chat::load_history(probe, snapshot);

    if restore_optimizer {
        probe.goal_optimizer.restore(snapshot);
    }
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
        probe.workspace_next_refresh =
            Some(std::time::Instant::now() + std::time::Duration::from_secs(30));
    }
}
