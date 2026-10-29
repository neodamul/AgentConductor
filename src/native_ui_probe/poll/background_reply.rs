use super::super::Probe;

fn take_poll_pending(
    foreground: Option<super::super::Pending>,
    poll: &mut Option<super::super::Pending>,
) -> Option<super::super::Pending> {
    if foreground.is_some() {
        None
    } else {
        poll.take()
    }
}

fn poll_reply_invalidated(event_arrived_during_poll: bool) -> bool {
    event_arrived_during_poll
}

#[cfg(test)]
#[path = "../../../tests/native_ui_probe/poll_background_reply.rs"]
mod tests;

impl Probe {
    pub(super) fn poll_background_reply(&mut self) {
        if self.pending.is_some() {
            return;
        }
        let reply = self
            .worker
            .as_ref()
            .map(|worker| worker.poll_replies.try_recv());
        match reply {
            Some(Ok(reply)) => {
                let request_revision = self.poll_request_revision.take();
                if self.worker.as_ref().is_some_and(|worker| {
                    request_revision.is_some_and(|revision| revision != worker.latest_request_id())
                }) {
                    self.poll_pending = None;
                    return;
                }
                let invalidated = self
                    .worker
                    .as_ref()
                    .is_some_and(|worker| worker.events.try_recv().is_ok());
                if poll_reply_invalidated(invalidated) {
                    self.poll_pending = None;
                    self.refresh_background();
                    return;
                }
                let Some(pending) = take_poll_pending(self.pending, &mut self.poll_pending) else {
                    return;
                };
                self.pending = Some(pending);
                self.apply_reply(Ok(reply));
            }
            Some(Err(std::sync::mpsc::TryRecvError::Disconnected)) => {
                self.poll_pending = None;
                self.poll_request_revision = None;
                self.error = Some("Background poll worker stopped".into());
            }
            Some(Err(std::sync::mpsc::TryRecvError::Empty)) | None => {}
        }
    }
}
