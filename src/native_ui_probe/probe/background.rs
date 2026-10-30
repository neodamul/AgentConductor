use super::Probe;

impl Probe {
    pub(in crate::native_ui_probe) fn refresh_background(&mut self) {
        if self.background_refresh_pending {
            return;
        }
        if self.worker.as_ref().is_some_and(|worker| {
            let request = if self.room_view.is_some() {
                br#"{"kind":"snapshot","icons":true}"#.as_slice()
            } else {
                super::super::worker::initial_snapshot(false)
            };
            worker.send_background(request).is_ok()
        }) {
            self.background_refresh_pending = true;
            self.background_refresh_generation = self.refresh_generation;
            self.refresh_requested = false;
        }
    }
}
