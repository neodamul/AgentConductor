use super::{Pending, Probe};
use crate::native_ui_probe::room::task_controls::{self, Action};

impl Probe {
    pub(in crate::native_ui_probe) fn send_task_action(&mut self, action: Action) {
        let resume = matches!(&action, Action::Feedback { .. })
            && self
                .snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot["report"]["state"] != "running");
        let coordinator_request = matches!(&action, Action::CoordinatorRequest { .. });
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Agent Room snapshot is unavailable".to_owned())
            .and_then(|snapshot| task_controls::request(snapshot, action))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Agent Room controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                if let Some(view) = self.room_view.as_mut()
                    && coordinator_request
                {
                    view.task_controls.coordinator_feedback.clear();
                }
                self.resume_after_task_feedback = resume;
                self.pending = Some(Pending::Action);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }
}
