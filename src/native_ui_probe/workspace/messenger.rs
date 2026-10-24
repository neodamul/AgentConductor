mod view;
use serde_json::Value;

#[derive(Debug, Eq, PartialEq)]
pub enum Event {
    Save(Vec<u8>),
    Test(Vec<u8>),
}

#[derive(Default)]
pub struct State {
    open: bool,
    settings: crate::messenger::Settings,
    url: String,
    token: String,
    relay_url: String,
    relay_secret: String,
    approvers: String,
    has_url: bool,
    has_token: bool,
    has_relay: bool,
    has_relay_secret: bool,
    notice: Option<String>,
}

impl State {
    pub fn open(&mut self, snapshot: &Value) {
        self.settings =
            serde_json::from_value(snapshot["config"]["messenger"].clone()).unwrap_or_default();
        self.approvers = self.settings.approvers.join(", ");
        self.url.clear();
        self.token.clear();
        self.relay_url.clear();
        self.relay_secret.clear();
        self.notice = None;
        self.open = true;
    }

    pub(in crate::native_ui_probe) fn apply_status(&mut self, status: &Value) {
        self.settings = serde_json::from_value(status["settings"].clone()).unwrap_or_default();
        self.has_url = status["has_url"] == true;
        self.has_token = status["has_token"] == true;
        self.has_relay = status["has_relay"] == true;
        self.has_relay_secret = status["has_relay_secret"] == true;
    }

    pub(in crate::native_ui_probe) fn saved(&mut self) {
        self.open = false;
    }

    pub(in crate::native_ui_probe) fn test_sent(&mut self, language: &str) {
        self.notice = Some(text(language, "startup.test_message_sent").into());
    }

    fn secrets(&self) -> [Option<String>; 4] {
        [
            optional(&self.url),
            optional(&self.token),
            optional(&self.relay_url),
            optional(&self.relay_secret),
        ]
    }
}

fn optional(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then(|| value.trim().to_owned())
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

impl super::super::Probe {
    pub(super) fn open_messenger(&mut self) {
        let Some(snapshot) = self.snapshot.as_ref() else {
            return;
        };
        self.messenger_editor.open(snapshot);
        match self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"messenger"}"#))
        {
            Ok(()) => {
                self.pending = Some(super::super::Pending::MessengerLoad);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn send_messenger(&mut self, event: Event) {
        let (packet, pending) = match event {
            Event::Save(packet) => (packet, super::super::Pending::MessengerSave),
            Event::Test(packet) => (packet, super::super::Pending::MessengerTest),
        };
        match self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(&packet))
        {
            Ok(()) => {
                self.pending = Some(pending);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }
}

pub fn test_request(
    settings: &crate::messenger::Settings,
    url: Option<String>,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&serde_json::json!({"kind":"test_messenger","settings":settings,"url":url}))
        .map_err(|error| error.to_string())
}

pub fn save_request(
    snapshot: &Value,
    settings: &crate::messenger::Settings,
    secrets: [Option<String>; 4],
) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .ok_or("workspace revision is unavailable")?;
    let [url, token, relay_url, relay_secret] = secrets;
    serde_json::to_vec(&serde_json::json!({
        "kind":"save_messenger","revision":revision,"settings":settings,
        "url":url,"token":token,"relay_url":relay_url,"relay_secret":relay_secret
    }))
    .map_err(|error| error.to_string())
}
