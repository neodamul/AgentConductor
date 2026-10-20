use super::{Pending, Probe, cache, room, workspace};

mod errors;
mod mcp;
mod settings;
mod workspace_ops;
use std::sync::mpsc;

impl Probe {
    pub(super) fn poll(&mut self) {
        let Some(worker) = &self.worker else { return };
        let reply = worker.replies.try_recv();
        match reply {
            Ok(Ok(snapshot)) => match self.pending {
                Some(
                    Pending::PickFolder
                    | Pending::ShareFolder(_)
                    | Pending::ShareExport
                    | Pending::SharePreview
                    | Pending::ShareImport
                    | Pending::PickExecutable
                    | Pending::ResolveExecutable
                    | Pending::ProfileSave
                    | Pending::CatalogSave
                    | Pending::CatalogRefresh
                    | Pending::ProfileRefresh
                    | Pending::ProfileRetryRefresh
                    | Pending::McpSave
                    | Pending::McpRefresh
                    | Pending::MessengerLoad
                    | Pending::MessengerTest
                    | Pending::MessengerSave
                    | Pending::MessengerRefresh
                    | Pending::Update
                    | Pending::Save
                    | Pending::Prepare
                    | Pending::NextPhase
                    | Pending::SelectGoal
                    | Pending::Metadata
                    | Pending::MetadataRefresh,
                ) => settings::apply(self, snapshot),
                Some(Pending::Action) => {
                    match room::action_reply(self.worker.as_ref().unwrap(), &snapshot) {
                        Ok(message) => {
                            if let Some(view) = self.room_view.as_mut() {
                                view.notice = message;
                            }
                            self.pending = Some(Pending::Refresh);
                        }
                        Err(error) => {
                            self.resume_after_task_feedback = false;
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
                Some(Pending::StopPlanning | Pending::StopPlanningRefresh) => {
                    workspace_ops::stop_plan_reply(self, &snapshot)
                }
                Some(Pending::NavigationRefresh) => {
                    workspace_ops::navigation_refresh(self, snapshot)
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
                Some(
                    Pending::McpDiscover
                    | Pending::McpPreview
                    | Pending::McpApply
                    | Pending::McpApplyRefresh
                    | Pending::McpStatus,
                ) => mcp::apply(self, snapshot),
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
                    let mut view = room::State::default();
                    view.workspace_available = true;
                    self.room_view = Some(view);
                    if let Some(worker) = self.worker.as_mut() {
                        worker.set_room_view(true);
                    }
                    self.starting_run = None;
                    self.workspace_next_refresh = None;
                    let result = self
                        .worker
                        .as_ref()
                        .ok_or_else(|| "Agent Room controller is unavailable".to_owned())
                        .and_then(|worker| {
                            worker.send(br#"{"kind":"snapshot","room":true,"icons":true}"#)
                        });
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
                    self.task_saved_notice = true;
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
                Some(Pending::TaskDetails) => {
                    if let Some(view) = self.room_view.as_mut() {
                        if snapshot["task"]["id"].as_str() == view.details_task.as_deref() {
                            view.task_details = Some(snapshot);
                            self.pending = None;
                            self.error = None;
                        } else {
                            self.pending = None;
                            self.error =
                                Some("Task activity response did not match the selection".into());
                        }
                    } else {
                        self.pending = None;
                        self.error = Some("Agent Room is unavailable".into());
                    }
                }
                Some(Pending::Refresh | Pending::PrepareRefresh) | None => {
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
                        workspace_ops::load_workspace(self, &snapshot);
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
                    if self.resume_after_task_feedback
                        && self
                            .snapshot
                            .as_ref()
                            .is_some_and(|s| s["job"]["kind"] == "revise")
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
                        self.resume_after_task_feedback = false;
                        if resume {
                            self.resume_room();
                        }
                    }
                    if self.pending.is_none() && self.room_view.is_some() {
                        self.refresh_selected_task_details();
                    }
                }
            },
            Ok(Err(error)) => self.handle_error(error),
            Err(mpsc::TryRecvError::Empty) => {}
            Err(error) => self.handle_error(error),
        }
    }
}
