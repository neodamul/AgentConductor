use super::{Pending, Probe};

impl Probe {
    pub(in crate::native_ui_probe) fn refresh_workspace(&mut self) {
        let request: &[u8] = if self.coordinator_chat.is_uncertain() {
            br#"{"kind":"snapshot","icons":true,"chat_history":true}"#
        } else {
            br#"{"kind":"snapshot","icons":true}"#
        };
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(request));
        match result {
            Ok(()) => {
                self.pending = Some(if self.navigation.stop_for_delete() {
                    Pending::StopPlanningRefresh
                } else if self.starting_run.is_some() {
                    Pending::StartRefresh
                } else {
                    Pending::Refresh
                })
            }
            Err(_) if self.coordinator_chat.is_uncertain() => {
                self.workspace_next_refresh =
                    Some(std::time::Instant::now() + std::time::Duration::from_millis(500));
            }
            Err(error) => self.error = Some(error),
        }
    }
}
