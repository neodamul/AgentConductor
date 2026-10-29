use super::super::Probe;

#[cfg(test)]
#[path = "../../../tests/native_ui_probe/poll_events.rs"]
mod tests;

impl Probe {
    pub(super) fn poll_events(&mut self) {
        if self.pending.is_some() || self.poll_pending.is_some() {
            return;
        }
        if self
            .room_view
            .as_ref()
            .is_some_and(super::super::room::State::blocks_background_refresh)
        {
            return;
        }
        let changed = self
            .worker
            .as_ref()
            .is_some_and(|worker| worker.events.try_recv().is_ok());
        if changed {
            self.refresh_invalidated_state();
        }
    }

    pub(super) fn refresh_invalidated_state(&mut self) {
        if self.starting_run.is_some() || self.navigation.stop_for_delete() {
            self.refresh_workspace();
        } else if self.coordinator_chat.awaiting() {
            if self.coordinator_chat.is_uncertain() {
                self.refresh_workspace();
            } else {
                self.refresh_coordinator_chat();
            }
        } else if self.goal_optimizer.awaiting() {
            self.refresh_goal_optimizer();
        } else if self
            .snapshot
            .as_ref()
            .is_some_and(super::super::workspace::profile_tools::metadata_running)
        {
            self.refresh_metadata();
        } else if self.mcp_editor.discovering() || self.mcp_sources.previewing() {
            self.refresh_mcp_snapshot();
        } else {
            self.refresh_background();
        }
    }
}
