use super::{Pending, Probe};
use eframe::egui::{self, RichText};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    List,
    Cache,
    Mcp,
    Messenger,
    Create(String),
    ConfirmCreate(String),
    Open(String),
    ConfirmOpen(String),
    Forget(String),
    Delete { path: String, name: String },
}

pub fn request(event: &Event) -> Vec<u8> {
    let value = match event {
        Event::List => serde_json::json!({"kind":"saved_workspaces"}),
        Event::Cache => serde_json::json!({"kind":"cache_preview"}),
        Event::Mcp | Event::Messenger => unreachable!("settings use their own requests"),
        Event::Create(name) | Event::ConfirmCreate(name) => {
            serde_json::json!({"kind":"create_workspace","name":name})
        }
        Event::Open(path) | Event::ConfirmOpen(path) => {
            serde_json::json!({"kind":"open_workspace","path":path})
        }
        Event::Forget(path) => serde_json::json!({"kind":"forget_workspace","path":path}),
        Event::Delete { path, name } => serde_json::json!({
            "kind":"delete_workspace", "path":path, "name":name, "replacement":null
        }),
    };
    serde_json::to_vec(&value).expect("workspace navigation request is serializable")
}

pub fn needs_discard_confirmation(event: &Event, dirty: bool) -> bool {
    dirty && matches!(event, Event::Create(_) | Event::Open(_))
}

#[derive(Clone)]
enum Confirmation {
    Create(String),
    Open { path: String, name: String },
    Delete { path: String, name: String },
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
}

impl State {
    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn set_workspaces(&mut self, workspaces: Vec<Value>) {
        self.workspaces = workspaces;
        self.loading = false;
    }

    pub fn confirm_delete(&mut self, path: &str, name: &str) {
        self.confirmation = Some(Confirmation::Delete {
            path: path.into(),
            name: name.into(),
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
    }
}

pub fn show(
    state: &mut State,
    current_name: &str,
    current_path: &str,
    can_change: bool,
    dirty: bool,
    show_cache: bool,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let mut event = None;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("AC")
                .strong()
                .color(egui::Color32::WHITE)
                .background_color(egui::Color32::from_rgb(10, 102, 216)),
        );
        if show_cache
            && ui
                .add_enabled(can_change, egui::Button::new("Cache cleanup"))
                .accessible_name("Preview and clear local cache")
                .clicked()
        {
            event = Some(Event::Cache);
        }
        ui.heading("AgentConductor");
        ui.label(
            RichText::new("NATIVE UI PROBE")
                .small()
                .color(egui::Color32::from_rgb(96, 96, 103)),
        );
        if ui
            .button(format!(
                "{} ▾ {current_name}",
                state.appearance.text("startup.workspace_373010")
            ))
            .accessible_name(state.appearance.text("startup.open_workspace_menu"))
            .clicked()
        {
            state.open = !state.open;
            if state.open {
                event = Some(Event::List);
            }
        }
        match crate::native_ui_probe::appearance::show(&mut state.appearance, ui) {
            Some(crate::native_ui_probe::appearance::SettingsAction::Mcp) => {
                event = Some(Event::Mcp);
            }
            Some(crate::native_ui_probe::appearance::SettingsAction::Messenger) => {
                event = Some(Event::Messenger);
            }
            None => {}
        }
    });
    if state.open {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.strong("Saved workspaces");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut state.name)
                    .on_hover_text("Name for a new workspace");
                if ui
                    .add_enabled(
                        can_change && !state.loading && !state.name.trim().is_empty(),
                        egui::Button::new("Create workspace"),
                    )
                    .accessible_name("Create workspace")
                    .clicked()
                {
                    let create = Event::Create(state.name.trim().to_owned());
                    if needs_discard_confirmation(&create, dirty) {
                        state.confirmation = Some(Confirmation::Create(state.name.trim().into()));
                    } else {
                        event = Some(create);
                    }
                }
            });
            if state.loading {
                ui.spinner();
            } else if state.workspaces.is_empty() {
                ui.label("No saved workspaces");
            }
            let mut confirmation = None;
            for workspace in &state.workspaces {
                let name = workspace["name"].as_str().unwrap_or("Workspace");
                let path = workspace["path"].as_str().unwrap_or_default();
                let current = path == current_path;
                ui.horizontal(|ui| {
                    ui.label(format!("{name} · {}", display_path(path)));
                    if current {
                        ui.label("Current");
                    } else if ui
                        .add_enabled(can_change && !state.loading, egui::Button::new("Open"))
                        .accessible_name(format!("Open workspace {name}"))
                        .clicked()
                    {
                        let open = Event::Open(path.into());
                        if needs_discard_confirmation(&open, dirty) {
                            confirmation = Some(Confirmation::Open {
                                path: path.into(),
                                name: name.into(),
                            });
                        } else {
                            event = Some(open);
                        }
                    }
                    if !current
                        && workspace["registered"] == true
                        && ui
                            .add_enabled(!state.loading, egui::Button::new("Remove from list"))
                            .accessible_name(format!("Remove {name} from list"))
                            .clicked()
                    {
                        event = Some(Event::Forget(path.into()));
                    }
                    if ui
                        .add_enabled(can_change && !state.loading, egui::Button::new("Archive"))
                        .accessible_name(format!("Archive workspace {name}"))
                        .clicked()
                    {
                        confirmation = Some(Confirmation::Delete {
                            path: path.into(),
                            name: name.into(),
                        });
                    }
                });
            }
            if confirmation.is_some() {
                state.confirmation = confirmation;
            }
        });
    }
    if let Some(confirm) = state.confirmation.clone() {
        egui::Frame::group(ui.style()).show(ui, |ui| match confirm {
            Confirmation::Create(name) => {
                ui.label(format!(
                    "Create {name} and discard unsaved workspace edits?"
                ));
                if ui.button("Confirm and switch").clicked() {
                    event = Some(Event::ConfirmCreate(name));
                    state.confirmation = None;
                }
                if ui.button("Cancel creation").clicked() {
                    state.confirmation = None;
                }
            }
            Confirmation::Open { path, name } => {
                ui.label(format!(
                    "Open {name} at {path}? Unsaved workspace edits will be discarded."
                ));
                if ui
                    .button(format!("Confirm open workspace {name}"))
                    .accessible_name(format!("Confirm open workspace {name}"))
                    .clicked()
                {
                    event = Some(Event::ConfirmOpen(path));
                    state.confirmation = None;
                }
                if ui.button(format!("Cancel open {name}")).clicked() {
                    state.confirmation = None;
                }
            }
            Confirmation::Delete { path, name } => {
                ui.label(format!(
                    "Archive workspace {name} at {}? Project source files are preserved.{}",
                    display_path(&path),
                    if dirty {
                        " Unsaved workspace edits will be discarded."
                    } else {
                        ""
                    }
                ))
                .accessible_name(format!(
                    "Archive workspace {name} at {path}; project source files are preserved"
                ));
                if ui
                    .button(format!("Confirm archive {name}"))
                    .accessible_name(format!("Confirm archive {name}"))
                    .clicked()
                {
                    event = Some(Event::Delete {
                        path: path.clone(),
                        name: name.clone(),
                    });
                    state.confirmation = None;
                }
                if ui
                    .button(format!("Cancel archive {name}"))
                    .accessible_name(format!("Cancel archive {name}"))
                    .clicked()
                {
                    state.confirmation = None;
                }
            }
        });
    }
    event
}

impl Probe {
    pub(super) fn show_navigation(&mut self, ui: &mut egui::Ui) {
        let snapshot = self.snapshot.as_ref();
        let name = snapshot
            .and_then(|value| value["config"]["name"].as_str())
            .unwrap_or("Workspace");
        let path = snapshot
            .and_then(|value| value["root"].as_str())
            .unwrap_or_default();
        let dirty = snapshot.is_some_and(|value| {
            value["config"]["name"] != self.name
                || value["config"]["repository"].as_str() != Some(self.repository.as_str())
                || value["config"]["goal"] != self.goal
                || value["config"]["workspace_mode"]
                    != serde_json::to_value(self.workspace_mode).unwrap_or_default()
                || value["config"]["coordinator"] != self.coordinator
                || value["config"]["profiles"].as_array() != Some(&self.profiles)
                || value["config"]["roles"] != serde_json::to_value(&self.roles).unwrap_or_default()
                || self.profile_editor.is_editing()
                || self
                    .task_draft
                    .as_ref()
                    .is_some_and(|draft| draft.changed(value))
        });
        let can_change = self.pending.is_none()
            && self.starting_run.is_none()
            && self.room_view.is_none()
            && snapshot.is_none_or(|value| value["job"]["state"] != "running");
        let event = show(
            &mut self.navigation,
            name,
            path,
            can_change,
            dirty,
            self.room_view.is_none(),
            ui,
        );
        match event {
            Some(Event::Mcp) => self.mcp_editor.open(self.snapshot.as_ref()),
            Some(Event::Messenger) => self.open_messenger(),
            Some(Event::Cache) => self.open_cache(),
            Some(event) => self.navigate(event),
            None => {}
        }
    }

    pub(in crate::native_ui_probe) fn navigate(&mut self, event: Event) {
        if event == Event::Cache {
            self.open_cache();
            return;
        }
        if self.pending.is_some() || (self.room_view.is_some() && event != Event::List) {
            return;
        }
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(&request(&event)));
        match result {
            Ok(()) => {
                self.navigation.begin(event);
                self.pending = Some(Pending::Navigation);
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
                    .and_then(|worker| worker.send(br#"{"kind":"snapshot"}"#));
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

fn display_path(path: &str) -> String {
    crate::git::git_path(std::path::Path::new(path).as_os_str())
        .to_string_lossy()
        .into_owned()
}
