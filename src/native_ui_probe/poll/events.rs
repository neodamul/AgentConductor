use super::super::Probe;

impl Probe {
    pub(super) fn poll_events(&mut self) {
        if self.pending.is_some() || self.poll_pending.is_some() {
            return;
        }
        let changed = self
            .worker
            .as_ref()
            .is_some_and(|worker| worker.events.try_recv().is_ok());
        if changed {
            if self
                .snapshot
                .as_ref()
                .is_some_and(super::super::workspace::profile_tools::metadata_running)
            {
                self.refresh_metadata();
            } else {
                self.refresh_background();
            }
        }
    }
}
