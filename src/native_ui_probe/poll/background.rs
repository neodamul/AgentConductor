use super::super::{Probe, room};

impl Probe {
    pub(super) fn poll_background(&mut self) {
        let Some(worker) = &self.worker else { return };
        match worker.background_replies.try_recv() {
            Ok(Ok(snapshot)) => {
                self.background_refresh_pending = false;
                self.poll_succeeded();
                self.refresh_requested =
                    self.refresh_generation != self.background_refresh_generation;
                if self.pending.is_some() {
                    self.refresh_requested = true;
                    return;
                }
                if let Some(view) = self.room_view.as_mut() {
                    if view
                        .evidence
                        .as_ref()
                        .is_some_and(|evidence| evidence["revision"] != snapshot["revision"])
                    {
                        view.evidence = None;
                        view.diff.clear();
                    }
                    room::refresh_follow_up::observe_snapshot(
                        view,
                        &snapshot,
                        std::time::Instant::now(),
                    );
                } else {
                    super::workspace_ops::load_workspace(self, &snapshot);
                }
                self.snapshot = Some(snapshot);
                self.error = None;
                super::super::workspace::plan_source::after_refresh(self);
                if self.room_view.is_some() {
                    self.refresh_room_details(true);
                }
            }
            Ok(Err(error)) => self.failed(error),
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.failed("Background refresh worker disconnected".to_owned())
            }
        }
    }

    fn failed(&mut self, error: String) {
        self.background_refresh_pending = false;
        self.refresh_requested = self.refresh_generation != self.background_refresh_generation;
        self.background_poll_failures = self.background_poll_failures.saturating_add(1);
        if self.background_poll_failures >= 3 && self.pending.is_none() {
            self.error = Some(error);
        }
    }
}
