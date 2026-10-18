use serde_json::Value;

use super::super::{Pending, Probe};

impl Probe {
    pub(super) fn refresh_after(&mut self, pending: Pending) {
        let result = self
            .worker
            .as_ref()
            .unwrap()
            .send(br#"{"kind":"snapshot"}"#);
        if let Err(error) = result {
            self.pending = None;
            self.error = Some(error);
        } else {
            self.pending = Some(pending);
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
