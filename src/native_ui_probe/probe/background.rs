use super::{Pending, Probe};

impl Probe {
    pub(in crate::native_ui_probe) fn refresh_background(&mut self) {
        if self
            .worker
            .as_ref()
            .is_some_and(|worker| worker.send(br#"{"kind":"snapshot","icons":true}"#).is_ok())
        {
            self.pending = Some(Pending::BackgroundRefresh);
        }
    }
}
