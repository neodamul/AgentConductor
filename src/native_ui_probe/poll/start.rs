use super::super::{Probe, workspace};
use serde_json::Value;
use std::time::{Duration, Instant};

impl Probe {
    pub(super) fn start_refresh_received(&mut self, snapshot: Value) {
        if self
            .starting_run
            .as_deref()
            .is_some_and(|run| workspace::execution_ready(&snapshot, run))
        {
            let result = self
                .worker
                .as_ref()
                .ok_or_else(|| "Workspace controller is unavailable".to_owned())
                .and_then(|worker| worker.send(br#"{"kind":"open_run"}"#));
            match result {
                Ok(()) => self.pending = Some(super::Pending::OpenRun),
                Err(error) => {
                    self.starting_run = None;
                    self.pending = None;
                    self.error = Some(error);
                }
            }
        } else if matches!(
            snapshot["job"]["state"].as_str(),
            Some("failed" | "interrupted" | "stopped")
        ) && snapshot["job"]["kind"] == "start"
        {
            self.starting_run = None;
            self.workspace_next_refresh = None;
            self.snapshot = Some(snapshot);
            self.pending = None;
            self.error = None;
        } else {
            self.workspace_next_refresh = Some(Instant::now() + Duration::from_secs(30));
            self.snapshot = Some(snapshot);
            self.pending = None;
            self.error = None;
        }
    }
}
