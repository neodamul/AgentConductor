use eframe::egui;
use serde_json::{Value, json};

#[derive(Default)]
pub struct Editor {
    open: bool,
    repository: String,
    default_repository: String,
    error: Option<String>,
}

pub enum Action {
    Save(String),
}

impl Editor {
    pub fn apply_settings(&mut self, settings: &Value) {
        self.repository = settings["repository"].as_str().unwrap_or_default().into();
        self.default_repository = settings["default_repository"]
            .as_str()
            .unwrap_or(crate::update_source::DEFAULT_REPOSITORY)
            .into();
        self.error = None;
        self.open = true;
    }

    pub fn saved(&mut self) {
        self.open = false;
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.open = true;
    }

    pub fn show(&mut self, language: &str, pending: bool, ui: &mut egui::Ui) -> Option<Action> {
        if !self.open {
            return None;
        }
        let text = |key| crate::ui::messages::in_language(language, key);
        let mut open = self.open;
        let mut save = false;
        let mut close_requested = false;
        egui::Window::new(text("startup.update_repository"))
            .id(egui::Id::NULL.with("update-repository-settings"))
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .default_width(520.0)
            .show(ui.ctx(), |ui| {
                ui.label(text("startup.update_repository_description"));
                ui.add(
                    egui::TextEdit::singleline(&mut self.repository)
                        .id(egui::Id::NULL.with("update-repository-input"))
                        .hint_text(format!(
                            "{} · {}",
                            text("startup.update_repository_hint"),
                            self.default_repository
                        ))
                        .desired_width(f32::INFINITY),
                )
                .accessible_name(text("startup.update_repository_input"));
                if let Some(error) = &self.error {
                    ui.colored_label(ui.visuals().error_fg_color, error)
                        .accessible_name(error.clone());
                }
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!pending, egui::Button::new(text("startup.save")))
                        .clicked()
                    {
                        save = true;
                    }
                    if ui.button(text("startup.cancel")).clicked() {
                        close_requested = true;
                    }
                    if pending {
                        ui.spinner();
                        ui.label(text("startup.saving_update_repository"));
                    }
                });
            });
        self.open = open && !close_requested;
        save.then(|| Action::Save(self.repository.trim().to_owned()))
    }
}

impl super::super::Probe {
    pub(in crate::native_ui_probe) fn open_update_source(&mut self) {
        if self.pending.is_some() {
            return;
        }
        match self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| {
                worker.send(
                    &serde_json::to_vec(&json!({"kind":"update_settings"}))
                        .expect("update settings request is serializable"),
                )
            }) {
            Ok(()) => {
                self.pending = Some(super::super::Pending::UpdateSourceLoad);
                self.error = None;
            }
            Err(error) => self.update_source.set_error(error),
        }
    }

    pub(in crate::native_ui_probe) fn show_update_source(&mut self, ui: &mut egui::Ui) {
        let language = self.navigation.appearance.language().to_owned();
        let pending = matches!(
            self.pending,
            Some(super::super::Pending::UpdateSourceLoad | super::super::Pending::UpdateSourceSave)
        );
        if let Some(Action::Save(repository)) = self.update_source.show(&language, pending, ui) {
            match self
                .worker
                .as_ref()
                .ok_or_else(|| "Workspace controller is unavailable".to_owned())
                .and_then(|worker| {
                    worker.send(
                        &serde_json::to_vec(
                            &json!({"kind":"set_update_settings","repository":repository}),
                        )
                        .expect("update source request is serializable"),
                    )
                }) {
                Ok(()) => {
                    self.pending = Some(super::super::Pending::UpdateSourceSave);
                    self.error = None;
                }
                Err(error) => self.update_source.set_error(error),
            }
        }
    }
}
