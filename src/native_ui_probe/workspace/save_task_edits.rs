use super::{Pending, Probe, task_edit_request};
use serde_json::Value;

impl Probe {
    pub(super) fn save_task_edits(
        &mut self,
        tasks: Vec<Value>,
        mode: crate::plan::WorkspaceMode,
        digest: &str,
    ) {
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Workspace proposal is unavailable".to_owned())
            .and_then(|snapshot| task_edit_request(snapshot, &tasks, mode, digest))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(Pending::Edit);
            }
            Err(error) => self.error = Some(error),
        }
    }
}
