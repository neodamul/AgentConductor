use super::{Pending, Probe};

impl Probe {
    pub(in crate::native_ui_probe) fn send_poll(
        &mut self,
        packet: &[u8],
        pending: Pending,
    ) -> Result<bool, String> {
        if self.poll_pending.is_some() {
            return Ok(false);
        }
        let worker = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())?;
        let request_revision = worker.latest_request_id();
        let sent = worker.send_poll(packet)?;
        if sent {
            self.poll_pending = Some(pending);
            self.poll_request_revision = Some(request_revision);
        }
        Ok(sent)
    }

    pub(in crate::native_ui_probe) fn refresh_background(&mut self) {
        let _ = self.send_poll(
            br#"{"kind":"snapshot","icons":true}"#,
            Pending::BackgroundRefresh,
        );
    }
}
