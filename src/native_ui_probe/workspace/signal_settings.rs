mod view;

use crate::cooperative::route::{Preference, Transport};
use serde_json::Value;

#[derive(Debug, Eq, PartialEq)]
pub enum Event {
    Save(Vec<u8>),
}

#[derive(Default)]
pub struct State {
    open: bool,
    loading: bool,
    loaded: bool,
    preference: Preference,
    port: String,
    error: Option<String>,
}

impl State {
    pub fn is_ready(&self) -> bool {
        self.open && self.loaded && !self.loading && self.error.is_none()
    }

    pub fn open(&mut self) {
        self.open = true;
        self.loading = true;
        self.loaded = false;
        self.error = None;
    }

    pub fn apply_status(&mut self, response: &Value) -> Result<(), String> {
        self.preference = serde_json::from_value(response["preference"].clone())
            .map_err(|error| format!("Invalid signal preferences: {error}"))?;
        self.port = self.preference.port.to_string();
        self.loading = false;
        self.loaded = true;
        self.error = None;
        Ok(())
    }

    pub fn failed(&mut self, error: impl ToString) {
        self.loading = false;
        self.error = Some(error.to_string());
    }

    pub fn saved(&mut self, response: &Value) -> Result<(), String> {
        self.apply_status(response)?;
        self.open = false;
        Ok(())
    }

    pub fn show(
        &mut self,
        language: &str,
        can_edit: bool,
        pending: bool,
        ui: &mut eframe::egui::Ui,
    ) -> Option<Event> {
        view::show(self, language, can_edit, pending, ui)
    }

    pub fn load_request() -> Vec<u8> {
        br#"{"kind":"signal_transport"}"#.to_vec()
    }

    pub fn save_request(&self) -> Result<Vec<u8>, String> {
        let mut preference = self.preference.clone();
        if let Ok(port) = self.port.parse::<u16>()
            && port != 0
        {
            preference.port = port;
        } else if preference.transport == Transport::Agentmemory {
            return Err("agentmemory port must be between 1 and 65535".into());
        }
        if preference.port == 0 {
            return Err("agentmemory port must be nonzero".into());
        }
        serde_json::to_vec(&serde_json::json!({
            "kind":"set_signal_transport", "preference":preference
        }))
        .map_err(|error| error.to_string())
    }
}

impl super::super::Probe {
    pub(in crate::native_ui_probe) fn open_signal_settings(&mut self) {
        if self.pending.is_some() {
            return;
        }
        self.signal_settings.open();
        match self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(&State::load_request()))
        {
            Ok(()) => {
                self.pending = Some(super::super::Pending::SignalSettingsLoad);
                self.error = None;
            }
            Err(error) => {
                self.signal_settings.failed(&error);
                self.error = Some(error);
            }
        }
    }

    pub(super) fn send_signal_settings(&mut self, event: Event) {
        let Event::Save(packet) = event;
        match self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(&packet))
        {
            Ok(()) => {
                self.pending = Some(super::super::Pending::SignalSettingsSave);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(in crate::native_ui_probe) fn signal_settings_reply(&mut self, response: Value) {
        let result = if let Some(error) = response["error"].as_str() {
            Err(error.to_owned())
        } else {
            match self.pending {
                Some(super::super::Pending::SignalSettingsLoad) => {
                    self.signal_settings.apply_status(&response)
                }
                Some(super::super::Pending::SignalSettingsSave) => {
                    self.signal_settings.saved(&response)
                }
                _ => return,
            }
        };
        self.pending = None;
        match result {
            Ok(()) => self.error = None,
            Err(error) => {
                self.signal_settings.failed(&error);
                self.error = Some(error);
            }
        }
    }
}
