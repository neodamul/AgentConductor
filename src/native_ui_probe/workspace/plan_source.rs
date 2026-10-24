use super::{Pending, Probe};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Import,
    Draft,
}

#[derive(Default)]
pub struct State {
    after_save: Option<Action>,
    picking_file: bool,
    active: Option<Action>,
}

impl State {
    pub fn is_picking_file(&self) -> bool {
        self.picking_file
    }
}

pub fn begin(probe: &mut Probe, action: Action, settings_current: bool) {
    if settings_current {
        start(probe, action);
        return;
    }
    probe.plan_source.after_save = Some(action);
    probe.send_workspace();
    if !matches!(probe.pending, Some(Pending::Save)) {
        cancel(probe);
    }
}

pub fn after_refresh(probe: &mut Probe) {
    if let Some(action) = probe.plan_source.after_save.take() {
        start(probe, action);
    }
}

pub fn picked_file(probe: &mut Probe, response: &Value) {
    probe.plan_source.picking_file = false;
    let Some(path) = response["picked_plan_file"]
        .as_str()
        .filter(|path| !path.is_empty())
    else {
        probe.pending = None;
        probe.error = None;
        return;
    };
    let Some(revision) = probe
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot["revision"].as_str())
    else {
        probe.pending = None;
        probe.error = Some("Workspace revision is unavailable".into());
        return;
    };
    probe.plan_source.active = Some(Action::Import);
    dispatch(probe, request("read_plan", revision, Some(path)));
}

pub fn cancel(probe: &mut Probe) {
    probe.plan_source.after_save = None;
    probe.plan_source.picking_file = false;
    probe.plan_source.active = None;
}

pub fn accepted(probe: &mut Probe) {
    if probe.plan_source.active.take().is_some() {
        probe.workspace_step = 2;
    }
}

fn start(probe: &mut Probe, action: Action) {
    if action == Action::Import {
        let result = probe
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.pick_plan_file());
        match result {
            Ok(()) => {
                probe.plan_source.picking_file = true;
                probe.pending = Some(Pending::PickFolder);
                probe.error = None;
            }
            Err(error) => probe.error = Some(error),
        }
        return;
    }
    probe.plan_source.active = Some(Action::Draft);
    let result = probe
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot["revision"].as_str())
        .ok_or_else(|| "Workspace revision is unavailable".to_owned())
        .map(|revision| request("draft", revision, None));
    match result {
        Ok(packet) => dispatch(probe, packet),
        Err(error) => {
            cancel(probe);
            probe.error = Some(error);
        }
    }
}

fn dispatch(probe: &mut Probe, packet: Vec<u8>) {
    let result = probe
        .worker
        .as_ref()
        .ok_or_else(|| "Workspace controller is unavailable".to_owned())
        .and_then(|worker| worker.send(&packet));
    match result {
        Ok(()) => {
            probe.error = None;
            probe.pending = Some(Pending::Prepare);
        }
        Err(error) => {
            cancel(probe);
            probe.pending = None;
            probe.error = Some(error);
        }
    }
}

fn request(kind: &str, revision: &str, path: Option<&str>) -> Vec<u8> {
    let mut packet = json!({"kind": kind, "revision": revision});
    if let Some(path) = path {
        packet["path"] = path.into();
    }
    serde_json::to_vec(&packet).expect("plan source request is serializable")
}

#[cfg(test)]
#[path = "../../../tests/native_ui_probe/plan_source.rs"]
mod tests;
