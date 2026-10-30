use super::super::Probe;

#[cfg(test)]
#[path = "../../../tests/native_ui_probe/poll_events.rs"]
mod tests;

impl Probe {
    pub(super) fn poll_events(&mut self) {
        let Some(worker) = &self.worker else { return };
        while let Ok(crate::ui::events::Event::StateChanged) = worker.events.try_recv() {
            self.refresh_generation = self.refresh_generation.saturating_add(1);
            self.refresh_requested = true;
        }
    }

    pub(super) fn dispatch_background_refresh(&mut self) {
        if !self.refresh_requested || self.background_refresh_pending {
            return;
        }
        self.refresh_background();
    }
}
