use super::super::{Pending, Probe};

impl Probe {
    pub(super) fn handle_error(&mut self, error: impl ToString) {
        let error = error.to_string();
        if matches!(self.pending, Some(Pending::Update)) {
            self.pending = None;
            self.updates.set_error(error);
            return;
        }
        if matches!(self.pending, Some(Pending::StartRefresh | Pending::OpenRun)) {
            self.starting_run = None;
            self.workspace_next_refresh = None;
        }
        if matches!(
            self.pending,
            Some(Pending::McpDiscover | Pending::McpStatus)
        ) {
            self.mcp_editor.discovery_failed(&error);
            self.workspace_next_refresh = None;
        }
        self.pending = None;
        self.navigation_failed();
        self.error = Some(error);
    }
}
