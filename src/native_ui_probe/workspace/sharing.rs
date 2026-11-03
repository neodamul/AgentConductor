//! Safe request builders for the existing workspace settings share workflow.
use eframe::egui;
use serde_json::Value;

mod view;
pub use view::show;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Export,
    Import,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    Export,
    Import,
    ConfirmImport,
    CancelImport,
}

enum ResultMessage {
    Imported,
    Exported(String),
}

#[derive(Default)]
pub struct State {
    pub(super) open: bool,
    folder: String,
    preview: Option<Value>,
    keep_verified: bool,
    result: Option<ResultMessage>,
}

impl State {
    pub(super) fn open(&mut self) {
        self.open = true;
    }

    pub fn set_folder(&mut self, folder: String) {
        self.folder = folder;
        self.preview = None;
        self.result = None;
    }

    pub fn set_preview(&mut self, preview: Value) {
        self.preview = Some(preview);
        self.keep_verified = false;
    }

    pub fn imported(&mut self) {
        self.preview = None;
        self.result = Some(ResultMessage::Imported);
    }

    pub fn exported(&mut self, response: &Value) {
        self.result = Some(ResultMessage::Exported(
            response["exported"].as_str().unwrap_or_default().into(),
        ));
    }
}

fn packet(value: serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&value).expect("workspace sharing request is serializable")
}

pub fn export_request(folder: &str, revision: &str) -> Vec<u8> {
    packet(serde_json::json!({"kind":"export_settings","folder":folder,"revision":revision}))
}

pub fn preview_request(folder: &str) -> Vec<u8> {
    packet(serde_json::json!({"kind":"preview_settings","folder":folder}))
}

pub fn import_request(folder: &str, revision: &str, keep_verified: bool) -> Vec<u8> {
    packet(serde_json::json!({
        "kind":"import_settings",
        "folder":folder,
        "revision":revision,
        "keep_verified":keep_verified
    }))
}

impl super::super::Probe {
    pub(super) fn show_sharing(&mut self, context: &egui::Context) {
        if !self.sharing.open {
            return;
        }
        let language = self.navigation.appearance.language().to_owned();
        let enabled = self.pending.is_none()
            && self.starting_run.is_none()
            && self.room_view.is_none()
            && self
                .snapshot
                .as_ref()
                .is_some_and(|s| s["job"]["state"] != "running");
        let dirty = self.snapshot.as_ref().is_some_and(|value| {
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
        let mut open = self.sharing.open;
        let mut event = None;
        egui::Window::new(crate::ui::messages::in_language(
            &language,
            "startup.settings",
        ))
        .open(&mut open)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(480.0)
        .collapsible(false)
        .show(context, |ui| {
            event = show(&mut self.sharing, &language, enabled, dirty, ui);
        });
        self.sharing.open = open && self.sharing.open;
        match event {
            Some(Event::Export) => self.pick_share_folder(Action::Export),
            Some(Event::Import) => self.pick_share_folder(Action::Import),
            Some(Event::ConfirmImport) => self.import_shared_settings(),
            Some(Event::CancelImport) => self.sharing.preview = None,
            None => {}
        }
    }

    fn pick_share_folder(&mut self, action: Action) {
        match self.worker.as_ref().map_or_else(
            || Err("Workspace controller is unavailable".to_owned()),
            |worker| {
                worker.pick_folder(match action {
                    Action::Export => crate::ui::picker::FolderPurpose::WorkspaceSettingsExport,
                    Action::Import => crate::ui::picker::FolderPurpose::WorkspaceSettingsImport,
                })
            },
        ) {
            Ok(()) => self.pending = Some(super::super::Pending::ShareFolder(action)),
            Err(error) => self.error = Some(error),
        }
    }

    pub(in crate::native_ui_probe) fn share_folder_reply(
        &mut self,
        response: Value,
        action: Action,
    ) {
        let Some(folder) = response["picked_folder"].as_str().filter(|p| !p.is_empty()) else {
            self.pending = None;
            return;
        };
        self.sharing.set_folder(folder.into());
        let packet = match action {
            Action::Export => self.snapshot.as_ref().map(|snapshot| {
                export_request(folder, snapshot["revision"].as_str().unwrap_or_default())
            }),
            Action::Import => Some(preview_request(folder)),
        };
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| {
                worker.send(&packet.ok_or_else(|| "Workspace revision is unavailable".to_owned())?)
            });
        match result {
            Ok(()) => {
                self.pending = Some(match action {
                    Action::Export => super::super::Pending::ShareExport,
                    Action::Import => super::super::Pending::SharePreview,
                })
            }
            Err(error) => {
                self.pending = None;
                self.error = Some(error);
            }
        }
    }

    fn import_shared_settings(&mut self) {
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Workspace revision is unavailable".to_owned())
            .and_then(|snapshot| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Workspace controller is unavailable".to_owned())
                    .and_then(|worker| {
                        worker.send(&import_request(
                            &self.sharing.folder,
                            snapshot["revision"].as_str().unwrap_or_default(),
                            self.sharing.keep_verified,
                        ))
                    })
            });
        match result {
            Ok(()) => self.pending = Some(super::super::Pending::ShareImport),
            Err(error) => self.error = Some(error),
        }
    }
}
