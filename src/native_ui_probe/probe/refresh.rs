use super::{Pending, Probe};

impl Probe {
    pub(in crate::native_ui_probe) fn refresh_workspace(&mut self) {
        let request: &[u8] = if self.coordinator_chat.is_uncertain() {
            br#"{"kind":"snapshot","icons":true,"chat_history":true}"#
        } else {
            br#"{"kind":"snapshot","icons":true}"#
        };
        let pending = if self.navigation.stop_for_delete() {
            Pending::StopPlanningRefresh
        } else if self.starting_run.is_some() {
            Pending::StartRefresh
        } else {
            Pending::Refresh
        };
        match self.send_poll(request, pending) {
            Ok(_) => {}
            Err(_) if self.coordinator_chat.is_uncertain() => {
                self.workspace_next_refresh =
                    Some(std::time::Instant::now() + std::time::Duration::from_millis(500));
            }
            Err(error) => self.error = Some(error),
        }
    }
}
