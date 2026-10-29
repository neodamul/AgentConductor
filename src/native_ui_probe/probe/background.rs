use super::{Pending, Probe};

impl Probe {
    pub(in crate::native_ui_probe) fn refresh_background(&mut self) {
        let _ = self.send_poll(
            br#"{"kind":"snapshot","icons":true}"#,
            Pending::BackgroundRefresh,
        );
    }
}
