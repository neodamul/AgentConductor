use super::super::{Pending, Probe};

impl Probe {
    pub(in crate::native_ui_probe) fn handle_error(&mut self, error: impl ToString) {
        let error = error.to_string();
        if matches!(self.pending, Some(Pending::BackgroundRefresh)) {
            self.background_poll_failures = self.background_poll_failures.saturating_add(1);
            if self.background_poll_failures >= 3 {
                self.error = Some(error);
            }
            self.pending = None;
            return;
        }
        if matches!(
            self.pending,
            Some(Pending::BackgroundTaskDetails | Pending::BackgroundCoordinator)
        ) {
            self.pending = None;
            return;
        }
        crate::native_ui_probe::workspace::plan_source::cancel(self);
        if matches!(
            self.pending,
            Some(Pending::ChatStart | Pending::ChatRefresh)
        ) {
            self.coordinator_chat.failed(error);
            self.pending = None;
            self.workspace_next_refresh = None;
            return;
        }
        if matches!(self.pending, Some(Pending::CatalogSave)) {
            self.profile_editor.set_catalog_error(Some(error));
            self.pending = None;
            return;
        }
        if matches!(self.pending, Some(Pending::ProfileSave)) && self.profile_editor.is_editing() {
            self.profile_editor
                .set_save_error(self.navigation.appearance.language(), &error);
            self.error = None;
            self.refresh_after(Pending::ProfileRetryRefresh);
            return;
        }
        if matches!(self.pending, Some(Pending::Update)) {
            self.pending = None;
            self.updates.set_error(error);
            return;
        }
        if matches!(
            self.pending,
            Some(Pending::UpdateSourceLoad | Pending::UpdateSourceSave)
        ) {
            self.pending = None;
            self.update_source.set_error(error);
            return;
        }
        if matches!(self.pending, Some(Pending::StartRefresh | Pending::OpenRun)) {
            self.starting_run = None;
            self.workspace_next_refresh = None;
        }
        if matches!(
            self.pending,
            Some(
                Pending::McpDiscover
                    | Pending::McpPreview
                    | Pending::McpApply
                    | Pending::McpApplyRefresh
                    | Pending::McpStatus
            )
        ) {
            self.mcp_editor.discovery_failed(&error);
            self.workspace_next_refresh = None;
        }
        self.pending = None;
        self.navigation_failed();
        self.error = Some(error);
    }

    pub(in crate::native_ui_probe) fn poll_succeeded(&mut self) {
        self.background_poll_failures = 0;
    }
}
