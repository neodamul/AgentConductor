use super::{Event, State};

impl State {
    pub(in crate::native_ui_probe) fn mark_stop_for_delete(&mut self) {
        self.stopping_for_delete = true;
    }

    pub(in crate::native_ui_probe) fn stop_for_delete(&self) -> bool {
        self.stopping_for_delete
    }

    pub(in crate::native_ui_probe) fn delete_after_stop(&self) -> Option<Event> {
        self.stopping_for_delete
            .then(|| self.action.clone())
            .flatten()
    }

    pub(in crate::native_ui_probe) fn finish_stop_for_delete(&mut self) {
        self.stopping_for_delete = false;
    }
}
