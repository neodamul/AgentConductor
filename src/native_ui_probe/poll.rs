use super::{Pending, Probe, cache, room, workspace};
use serde_json::Value;
use std::sync::mpsc;

impl Probe {
    pub(super) fn poll(&mut self) {
        let Some(worker) = &self.worker else { return };
        let reply = worker.replies.try_recv();
        match reply {
            Ok(Ok(snapshot)) => match self.pending {
                Some(Pending::PickFolder) => {
                    self.repository = workspace::picked_folder(&self.repository, &snapshot);
                    self.pending = None;
                    self.error = None;
                }
                Some(Pending::ShareFolder(action)) => self.share_folder_reply(snapshot, action),
                Some(Pending::ShareExport) => {
                    self.sharing.exported(&snapshot);
                    self.pending = None;
                    self.error = None;
                }
                Some(Pending::SharePreview) => {
                    if snapshot["settings"].is_object() {
                        self.sharing.set_preview(snapshot);
                        self.pending = None;
                        self.error = None;
                    } else {
                        self.pending = None;
                        self.error = Some("Workspace settings preview was not returned".into());
                    }
                }
                Some(Pending::ShareImport) if snapshot["imported"] == true => {
                    self.sharing.imported();
                    self.refresh_after(Pending::Refresh);
                }
                Some(Pending::ShareImport) => {
                    self.pending = None;
                    self.error = Some("Workspace settings were not imported".into());
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
                Some(Pending::ToolDecision { resume_after }) => {
                    match room::action_reply(self.worker.as_ref().unwrap(), &snapshot) {
                        Ok(message) => {
                            if let Some(view) = self.room_view.as_mut() {
                                view.notice = message;
                            }
                            self.resume_after_tool_approval = resume_after;
                            self.pending = Some(Pending::Refresh);
                        }
                        Err(error) => {
                            self.pending = None;
                            self.error = Some(error);
                        }
                    }
                }
                Some(Pending::Briefing) => {
                    if let Some(view) = self.room_view.as_mut() {
                        view.briefing = Some(snapshot);
                        self.pending = None;
                        self.error = None;
                    } else {
                        self.pending = None;
                        self.error = Some("Agent Room is unavailable".into());
                    }
                }
                Some(Pending::CachePreview) => {
                    cache::apply_preview(&mut self.cache, snapshot);
                    self.pending = None;
                    self.error = None;
                }
                Some(Pending::CacheClear) => {
                    cache::apply_clear(&mut self.cache, snapshot);
                    self.pending = None;
                    self.error = None;
                }
                Some(Pending::Navigation) => self.navigation_reply(snapshot),
                Some(Pending::NavigationRefresh) => {
                    load_workspace(self, &snapshot);
                    self.snapshot = Some(snapshot);
                    self.pending = None;
                    self.error = None;
                    self.navigate(crate::native_ui_probe::workspace::navigation::Event::List);
                }
                Some(Pending::Start) => {
                    if let Some(run) = snapshot["run"].as_str().filter(|run| !run.is_empty()) {
                        self.starting_run = Some(run.to_owned());
                        self.refresh_after(Pending::StartRefresh);
                    } else {
                        self.pending = None;
                        self.error = Some("Started run path was not returned".into());
                    }
                }
                Some(Pending::StartRefresh) => {
                    if self
                        .starting_run
                        .as_deref()
                        .is_some_and(|run| workspace::execution_ready(&snapshot, run))
                    {
                        let result = self
                            .worker
                            .as_ref()
                            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
                            .and_then(|worker| worker.send(br#"{"kind":"open_run"}"#));
                        match result {
                            Ok(()) => self.pending = Some(Pending::OpenRun),
                            Err(error) => {
                                self.starting_run = None;
                                self.pending = None;
                                self.error = Some(error);
                            }
                        }
                    } else if matches!(
                        snapshot["job"]["state"].as_str(),
                        Some("failed" | "interrupted" | "stopped")
                    ) && snapshot["job"]["kind"] == "start"
                    {
                        self.starting_run = None;
                        self.workspace_next_refresh = None;
                        self.snapshot = Some(snapshot);
                        self.pending = None;
                        self.error = None;
                    } else {
                        self.workspace_next_refresh =
                            Some(std::time::Instant::now() + std::time::Duration::from_millis(500));
                        self.snapshot = Some(snapshot);
                        self.pending = None;
                        self.error = None;
                    }
                }
                Some(Pending::OpenRun) if snapshot["opened"] == true => {
                    self.room_view = Some(room::State::default());
                    self.starting_run = None;
                    self.workspace_next_refresh = None;
                    let result = self
                        .worker
                        .as_ref()
                        .ok_or_else(|| "Agent Room controller is unavailable".to_owned())
                        .and_then(|worker| worker.send(br#"{"kind":"snapshot","room":true}"#));
                    match result {
                        Ok(()) => {
                            self.pending = Some(Pending::Refresh);
                            self.error = None;
                        }
                        Err(error) => {
                            self.pending = None;
                            self.error = Some(error);
                        }
                    }
                }
                Some(Pending::OpenRun) => {
                    self.starting_run = None;
                    self.workspace_next_refresh = None;
                    self.pending = None;
                    self.error = Some("The approved run could not be opened".into());
                }
                Some(Pending::Edit) if snapshot["saved"] == true => {
                    self.task_draft = None;
                    self.refresh_after(Pending::Refresh)
                }
                Some(Pending::Edit) => {
                    self.pending = None;
                    self.error = Some("Edited plan was not saved".into());
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
                        if view
                            .evidence
                            .as_ref()
                            .is_some_and(|evidence| evidence["revision"] != snapshot["revision"])
                        {
                            view.evidence = None;
                            view.diff.clear();
                        }
                    } else {
                        load_workspace(self, &snapshot);
                    }
                    self.snapshot = Some(snapshot);
                    self.pending = None;
                    self.error = None;
                    if self.resume_after_tool_approval
                        && self
                            .snapshot
                            .as_ref()
                            .is_some_and(|s| s["job"]["kind"] == "tool_approve")
                        && self
                            .snapshot
                            .as_ref()
                            .is_some_and(|s| s["job"]["state"] != "running")
                    {
                        let resume = self.snapshot.as_ref().is_some_and(|s| {
                            s["job"]["state"] != "failed"
                                && matches!(
                                    s["report"]["state"].as_str(),
                                    Some("ready" | "stopped")
                                )
                        });
                        self.resume_after_tool_approval = false;
                        if resume {
                            self.resume_room();
                        }
                    }
                }
            },
            Ok(Err(error)) => {
                if matches!(self.pending, Some(Pending::StartRefresh | Pending::OpenRun)) {
                    self.starting_run = None;
                    self.workspace_next_refresh = None;
                }
                self.pending = None;
                self.navigation_failed();
                self.error = Some(error);
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(error) => {
                if matches!(self.pending, Some(Pending::StartRefresh | Pending::OpenRun)) {
                    self.starting_run = None;
                    self.workspace_next_refresh = None;
                }
                self.pending = None;
                self.navigation_failed();
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
