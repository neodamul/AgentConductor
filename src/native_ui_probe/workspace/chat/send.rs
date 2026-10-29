use super::super::super::{Pending, Probe};

impl Probe {
    pub(in crate::native_ui_probe) fn send_coordinator_chat(
        &mut self,
        turns: Vec<crate::chat::Turn>,
    ) {
        let packet = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Workspace revision is unavailable".to_owned())
            .and_then(|snapshot| super::request(snapshot, &turns));
        let packet = match packet {
            Ok(packet) => packet,
            Err(error) => {
                self.coordinator_chat.rejected(error);
                self.workspace_next_refresh = None;
                return;
            }
        };
        let Some(worker) = self.worker.as_ref() else {
            self.coordinator_chat
                .rejected("Workspace controller is unavailable".into());
            self.workspace_next_refresh = None;
            return;
        };
        match worker.send(&packet) {
            Ok(()) => {
                self.pending = Some(Pending::ChatStart);
                self.error = None;
            }
            Err(error) => {
                self.coordinator_chat.outcome_unknown(error);
                self.workspace_next_refresh =
                    Some(std::time::Instant::now() + std::time::Duration::from_millis(500));
                self.pending = None;
                self.error = None;
            }
        }
    }

    pub(in crate::native_ui_probe) fn refresh_coordinator_chat(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"snapshot","chat_result":true}"#));
        match result {
            Ok(()) => self.pending = Some(Pending::ChatRefresh),
            Err(error) => {
                self.coordinator_chat.failed(error);
                self.workspace_next_refresh = None;
            }
        }
    }
}
