use super::{Pending, Probe, room, workspace};
use serde_json::Value;
use std::sync::mpsc;

impl Probe {
    pub(super) fn poll(&mut self) {
        let Some(worker) = &self.worker else { return };
        let reply = worker.replies.try_recv();
        match reply {
            Ok(Ok(snapshot)) => {
                match self.pending {
                    Some(Pending::PickFolder) => {
                        self.repository = workspace::picked_folder(&self.repository, &snapshot);
                        self.pending = None;
                        self.error = None;
                    }
                    Some(Pending::PickExecutable) => {
                        if let Some(path) = snapshot["picked_executable"].as_str() {
                            self.profile_editor.set_executable(path);
                        }
                        self.pending = None;
                    }
                    Some(Pending::ResolveExecutable) => {
                        if let Some(requested) = self.executable_resolution.take() {
                            self.profile_editor
                                .apply_resolution(requested.as_str(), snapshot["path"].as_str());
                        }
                        self.pending = None;
                    }
                    Some(Pending::ProfileSave) if snapshot["saved"] == true => {
                        self.refresh_after(Pending::ProfileRefresh)
                    }
                    Some(Pending::ProfileSave) => {
                        self.pending = None;
                        self.error = Some("Agent profile was not saved".into());
                    }
                    Some(Pending::ProfileRefresh) => {
                        self.profile_editor.finish_save();
                        self.profiles = snapshot["config"]["profiles"]
                            .as_array()
                            .cloned()
                            .unwrap_or_default();
                        self.snapshot = Some(snapshot);
                        self.pending = None;
                        self.error = None;
                    }
                    Some(Pending::Save) if snapshot["saved"] == true => {
                        self.refresh_after(Pending::Refresh)
                    }
                    Some(Pending::Save) => {
                        self.pending = None;
                        self.error = Some("Workspace goal was not saved".into());
                    }
                    Some(Pending::Prepare) => self.refresh_after(Pending::Refresh),
                    Some(Pending::Metadata) => self.refresh_after(Pending::MetadataRefresh),
                    Some(Pending::MetadataRefresh) => {
                        if let Some(saved) = snapshot["config"]["profiles"].as_array() {
                            workspace::profile_tools::merge_catalogs(&mut self.profiles, saved);
                        }
                        self.snapshot = Some(snapshot);
                        self.pending = None;
                        self.error = None;
                    }
                    Some(Pending::Action) => {
                        match room::action_reply(self.worker.as_ref().unwrap(), &snapshot) {
                            Ok(message) => {
                                if let Some(view) = self.room_view.as_mut() {
                                    view.notice = message;
                                }
                                self.pending = Some(Pending::Refresh);
                            }
                            Err(error) => {
                                self.pending = None;
                                self.error = Some(error);
                            }
                        }
                    }
                    Some(Pending::Evidence) => {
                        let result = self
                            .room_view
                            .as_mut()
                            .ok_or_else(|| "Agent Room is unavailable".to_owned())
                            .and_then(|view| {
                                room::evidence_reply(view, self.snapshot.as_ref(), snapshot)
                            });
                        match result {
                            Ok(()) => {
                                self.pending = None;
                                self.error = None;
                            }
                            Err(error) => {
                                self.pending = None;
                                self.error = Some(error);
                            }
                        }
                    }
                    Some(Pending::Refresh) | None => {
                        if let Some(view) = self.room_view.as_mut() {
                            if view.evidence.as_ref().is_some_and(|evidence| {
                                evidence["revision"] != snapshot["revision"]
                            }) {
                                view.evidence = None;
                                view.diff.clear();
                            }
                        } else {
                            load_workspace(self, &snapshot);
                        }
                        self.snapshot = Some(snapshot);
                        self.pending = None;
                        self.error = None;
                    }
                }
            }
            Ok(Err(error)) => {
                self.pending = None;
                self.error = Some(error);
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(error) => {
                self.pending = None;
                self.error = Some(error.to_string());
            }
        }
    }
}

impl Probe {
    fn refresh_after(&mut self, pending: Pending) {
        let result = self
            .worker
            .as_ref()
            .unwrap()
            .send(br#"{"kind":"snapshot"}"#);
        if let Err(error) = result {
            self.pending = None;
            self.error = Some(error);
        } else {
            self.pending = Some(pending);
        }
    }
}

fn load_workspace(probe: &mut Probe, snapshot: &Value) {
    probe.goal = snapshot["config"]["goal"]
        .as_str()
        .unwrap_or_default()
        .into();
    probe.name = snapshot["config"]["name"]
        .as_str()
        .unwrap_or_default()
        .into();
    probe.repository = snapshot["config"]["repository"]
        .as_str()
        .unwrap_or_default()
        .into();
    probe.workspace_mode =
        serde_json::from_value(snapshot["config"]["workspace_mode"].clone()).unwrap_or_default();
    probe.profiles = snapshot["config"]["profiles"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    probe.coordinator = snapshot["config"]["coordinator"]
        .as_str()
        .unwrap_or_default()
        .into();
    probe.roles = serde_json::from_value(snapshot["config"]["roles"].clone()).unwrap_or_default();
}
