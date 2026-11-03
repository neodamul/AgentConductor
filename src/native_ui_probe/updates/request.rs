use super::super::{Pending, Probe};
use super::{Action, request};
use eframe::egui;

impl Probe {
    pub(in crate::native_ui_probe) fn open_updates(&mut self) {
        self.updates.open = true;
        if self.snapshot.is_some() && self.pending.is_none() && self.updates.next_poll.is_none() {
            self.update_request(Action::Check);
        }
    }

    pub(in crate::native_ui_probe) fn show_updates(&mut self, ui: &mut egui::Ui) {
        let action = self.updates.show(
            self.navigation.appearance.language(),
            self.pending.is_some(),
            ui,
        );
        if let Some(action) = action {
            self.update_request(action);
        }
    }

    pub(in crate::native_ui_probe) fn update_request(&mut self, action: Action) {
        if self.pending.is_some() {
            self.updates.set_error(
                self.navigation
                    .appearance
                    .text("native.update_already_in_progress")
                    .to_owned(),
            );
            return;
        }
        self.updates.requested(action);
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(&request(action)));
        match result {
            Ok(()) => {
                self.pending = Some(Pending::Update);
                self.error = None;
            }
            Err(error) => self.updates.set_error(error),
        }
    }
}
