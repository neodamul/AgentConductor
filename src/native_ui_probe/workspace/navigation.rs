use super::{Pending, Probe};
use eframe::egui;
use serde_json::Value;

mod action;
mod confirmation;
mod context;
mod helpers;
mod lifecycle;
mod probe_navigation;
pub mod stop;
pub mod tools_guide;
mod view;
mod view_menu;
pub use action::{Event, needs_discard_confirmation, request};
pub use context::NavigationContext;
use helpers::{display_path, message, text};
pub use view::show_in_language;

#[derive(Clone)]
enum Confirmation {
    Create(String),
    Open {
        path: String,
        name: String,
    },
    Delete {
        path: String,
        name: String,
    },
    StopAndDelete {
        path: String,
        name: String,
    },
    Discard {
        areas: Vec<String>,
        return_to_run: bool,
    },
}

#[derive(Default)]
pub struct State {
    pub(in crate::native_ui_probe) appearance: crate::native_ui_probe::appearance::State,
    open: bool,
    loading: bool,
    name: String,
    workspaces: Vec<Value>,
    confirmation: Option<Confirmation>,
    action: Option<Event>,
    stopping_for_delete: bool,
    tools_guide: bool,
    manual_open: bool,
    manual_language: String,
    manual_blocks: Vec<crate::manual::Block>,
    run_available: bool,
    dirty_areas: Vec<String>,
}

impl State {
    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn set_workspaces(&mut self, workspaces: Vec<Value>) {
        self.workspaces = workspaces;
        self.loading = false;
    }

    pub fn set_header_state(&mut self, run_available: bool, dirty_areas: Vec<String>) {
        self.run_available = run_available;
        self.dirty_areas = dirty_areas;
    }

    pub fn confirm_delete(&mut self, path: &str, name: &str) {
        self.confirmation = Some(Confirmation::Delete {
            path: path.into(),
            name: name.into(),
        });
    }

    pub fn confirm_discard(&mut self, areas: Vec<String>, return_to_run: bool) {
        self.confirmation = Some(Confirmation::Discard {
            areas,
            return_to_run,
        });
    }

    fn begin(&mut self, event: Event) {
        self.action = Some(event);
        self.loading = true;
    }

    fn list_finished(&mut self, response: &Value) -> bool {
        let Some(workspaces) = response["workspaces"].as_array() else {
            return false;
        };
        self.workspaces = workspaces.clone();
        self.action = None;
        self.loading = false;
        true
    }

    fn workspace_change_finished(&mut self) {
        self.stopping_for_delete = false;
        if matches!(
            self.action,
            Some(Event::Create(_) | Event::ConfirmCreate(_))
        ) {
            self.name.clear();
        }
        self.action = None;
        self.loading = false;
        self.open = false;
        self.confirmation = None;
    }

    fn forget_finished(&mut self) {
        self.action = None;
        self.loading = false;
    }

    fn failed(&mut self) {
        self.action = None;
        self.loading = false;
        self.stopping_for_delete = false;
    }
}

pub fn show(state: &mut State, context: NavigationContext<'_>, ui: &mut egui::Ui) -> Option<Event> {
    let language = state.appearance.language().to_owned();
    show_in_language(
        state,
        NavigationContext {
            language: &language,
            ..context
        },
        ui,
    )
}

impl Probe {
    pub(in crate::native_ui_probe) fn navigate(&mut self, event: Event) {
        if event == Event::Cache {
            self.open_cache();
            return;
        }
        if self.pending.is_some() || (self.room_view.is_some() && event != Event::List) {
            return;
        }
        let stop_plan = matches!(&event, Event::Delete { path, .. }
        if self.snapshot.as_ref().is_some_and(|snapshot| {
            snapshot["root"].as_str() == Some(path.as_str())
                && snapshot["job"]["kind"] == "plan"
                && snapshot["job"]["state"] == "running"
        }));
        let packet = if stop_plan {
            stop::request().to_vec()
        } else {
            request(&event)
        };
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(&packet));
        match result {
            Ok(()) => {
                self.navigation.begin(event);
                if stop_plan {
                    self.navigation.mark_stop_for_delete();
                    self.workspace_next_refresh =
                        Some(std::time::Instant::now() + std::time::Duration::from_millis(500));
                }
                self.pending = Some(if stop_plan {
                    Pending::StopPlanning
                } else {
                    Pending::Navigation
                });
                self.error = None;
            }
            Err(error) => {
                self.navigation.failed();
                self.error = Some(error);
            }
        }
    }

    pub(in crate::native_ui_probe) fn navigation_reply(&mut self, response: Value) {
        let event = self.navigation.action.clone();
        match event {
            Some(Event::List) => {
                if self.navigation.list_finished(&response) {
                    self.pending = None;
                    self.error = None;
                } else {
                    self.navigation.failed();
                    self.pending = None;
                    self.error = Some("Saved workspace list was not returned".into());
                }
            }
            Some(Event::Forget(_)) => {
                self.navigation.forget_finished();
                self.pending = None;
                self.navigate(Event::List);
            }
            Some(_) => {
                let current = self
                    .snapshot
                    .as_ref()
                    .and_then(|snapshot| snapshot["root"].as_str())
                    .unwrap_or_default();
                let changes_current = match &event {
                    Some(Event::Create(_) | Event::ConfirmCreate(_)) => true,
                    Some(Event::Open(path) | Event::ConfirmOpen(path)) => path != current,
                    Some(Event::Delete { path, .. }) => path == current,
                    _ => false,
                };
                if !changes_current {
                    self.navigation.forget_finished();
                    self.pending = None;
                    self.navigate(Event::List);
                    return;
                }
                self.navigation.workspace_change_finished();
                self.profile_editor.cancel();
                self.task_draft = None;
                self.confirming_start = None;
                self.workspace_step = 0;
                let refresh = self
                    .worker
                    .as_ref()
                    .ok_or_else(|| "Workspace controller is unavailable".to_owned())
                    .and_then(|worker| worker.send(br#"{"kind":"snapshot","icons":true}"#));
                match refresh {
                    Ok(()) => self.pending = Some(Pending::NavigationRefresh),
                    Err(error) => {
                        self.pending = None;
                        self.error = Some(error);
                    }
                }
            }
            None => {
                self.pending = None;
                self.error = Some("Workspace action state was lost".into());
            }
        }
    }

    pub(in crate::native_ui_probe) fn navigation_failed(&mut self) {
        self.navigation.failed();
    }
}
