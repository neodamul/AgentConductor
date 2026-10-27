use super::{Event, NavigationContext, Probe, show};
use eframe::egui;
use serde_json::Value;

impl Probe {
    pub(in crate::native_ui_probe::workspace) fn show_navigation(&mut self, ui: &mut egui::Ui) {
        let snapshot = self.snapshot.as_ref();
        let name = snapshot
            .and_then(|value| value["config"]["name"].as_str())
            .unwrap_or_else(|| {
                self.navigation
                    .appearance
                    .text("startup.workspace_fallback")
            });
        let path = snapshot
            .and_then(|value| value["root"].as_str())
            .unwrap_or_default();
        let language = self.navigation.appearance.language().to_owned();
        let dirty_areas = if self.room_view.is_none() {
            self.dirty_areas(snapshot, &language)
        } else {
            Vec::new()
        };
        let dirty = !dirty_areas.is_empty();
        self.navigation.set_header_state(
            self.room_view.is_none()
                && snapshot.is_some_and(|value| value["run_available"] == true),
            dirty_areas,
        );
        let can_change = self.pending.is_none()
            && self.starting_run.is_none()
            && self.room_view.is_none()
            && snapshot.is_none_or(|value| value["job"]["state"] != "running");
        let event = show(
            &mut self.navigation,
            NavigationContext {
                current_name: name,
                current_path: path,
                can_change,
                planning_running: snapshot.is_some_and(|value| {
                    value["job"]["kind"] == "plan" && value["job"]["state"] == "running"
                }),
                dirty,
                show_cache: self.room_view.is_none(),
                language: "en",
            },
            ui,
        );
        match event {
            Some(Event::Mcp) => self.mcp_editor.open(self.snapshot.as_ref()),
            Some(Event::Messenger) => self.open_messenger(),
            Some(Event::SignalTransport) => self.open_signal_settings(),
            Some(Event::WorkspaceSharing) => self.sharing.open(),
            Some(Event::UpdateRepository) => self.open_update_source(),
            Some(Event::Updates) => self.open_updates(),
            Some(Event::Cache) => self.open_cache(),
            Some(Event::BackToRun) => self.open_run(),
            Some(Event::DiscardChanges { return_to_run }) => {
                self.discard_workspace_changes();
                if return_to_run {
                    self.open_run();
                }
            }
            Some(event) => self.navigate(event),
            None => {}
        }
    }

    fn dirty_areas(&self, snapshot: Option<&Value>, language: &str) -> Vec<String> {
        let Some(snapshot) = snapshot else {
            return Vec::new();
        };
        let config = &snapshot["config"];
        let workspace_dirty = config["name"] != self.name
            || config["repository"].as_str() != Some(self.repository.as_str())
            || config["goal"] != self.goal
            || config["workspace_mode"]
                != serde_json::to_value(self.workspace_mode).unwrap_or_default();
        let profiles_dirty = config["coordinator"] != self.coordinator
            || config["profiles"].as_array() != Some(&self.profiles)
            || config["roles"] != serde_json::to_value(&self.roles).unwrap_or_default()
            || self.profile_editor.is_editing();
        let tasks_dirty = self
            .task_draft
            .as_ref()
            .is_some_and(|draft| draft.changed(snapshot));
        let mut areas = Vec::new();
        for (dirty, key) in [
            (workspace_dirty, "startup.workspace_settings_5ebc59"),
            (profiles_dirty, "startup.agent_profiles"),
            (tasks_dirty, "startup.plan_tasks"),
        ] {
            if dirty {
                areas.push(crate::ui::messages::in_language(language, key).to_owned());
            }
        }
        areas
    }

    fn discard_workspace_changes(&mut self) {
        if let Some(snapshot) = &self.snapshot {
            let config = &snapshot["config"];
            self.name = config["name"].as_str().unwrap_or_default().into();
            self.repository = config["repository"].as_str().unwrap_or_default().into();
            self.goal = config["goal"].as_str().unwrap_or_default().into();
            self.workspace_mode =
                serde_json::from_value(config["workspace_mode"].clone()).unwrap_or_default();
            self.coordinator = config["coordinator"].as_str().unwrap_or_default().into();
            self.profiles = config["profiles"].as_array().cloned().unwrap_or_default();
            self.roles = serde_json::from_value(config["roles"].clone()).unwrap_or_default();
        }
        self.profile_editor.cancel();
        self.task_draft = None;
        self.task_saved_notice = false;
        self.confirming_start = None;
        crate::native_ui_probe::workspace::plan_source::cancel(self);
        self.navigation.name.clear();
        self.workspace_step = 0;
        self.error = None;
    }

    fn open_run(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"open_run"}"#));
        match result {
            Ok(()) => {
                self.pending = Some(super::Pending::OpenRun);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/native_ui_probe/navigation_discard.rs"]
mod tests;
