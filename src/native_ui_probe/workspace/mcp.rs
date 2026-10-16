use eframe::egui::{self, TextEdit};
use serde_json::{Value, json};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    Save(String),
}

#[derive(Default)]
pub struct State {
    open: bool,
    settings: String,
    show_examples: bool,
}

impl State {
    pub(super) fn open(&mut self, snapshot: Option<&Value>) {
        let Some(snapshot) = snapshot else { return };
        self.show_examples = has_no_mcp_servers(snapshot);
        self.settings = settings_text(snapshot);
        self.open = true;
    }

    pub(in crate::native_ui_probe) fn saved(&mut self) {
        self.open = false;
    }

    pub(super) fn show(
        &mut self,
        snapshot: Option<&Value>,
        language: &str,
        can_save: bool,
        ui: &mut egui::Ui,
    ) -> Option<Event> {
        if !self.open {
            return None;
        }
        let text = |id| crate::ui::messages::in_language(language, id);
        let mut event = None;
        egui::Window::new(text("mcp.mcp_server_registration"))
            .open(&mut self.open)
            .default_width(620.0)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.heading(text("mcp.register_mcp_servers"));
                ui.label(text("mcp.paste_opencode_json_jsonc_with_an_mcp_map"));
                ui.label(text("mcp.server_definitions_and_optional_source_bindings"));
                if self.show_examples {
                    ui.label(text(
                        "mcp.examples_are_commented_replace_placeholders_and_remove_comments",
                    ));
                }
                ui.add_sized(
                    [ui.available_width(), 260.0],
                    TextEdit::multiline(&mut self.settings)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(14),
                );
                ui.label(text(
                    "mcp.local_commands_run_only_when_you_request_discovery",
                ));
                if ui
                    .add_enabled(
                        can_save && snapshot.is_some() && self.settings.len() < 64 * 1024,
                        egui::Button::new(text("mcp.save_mcp_settings")),
                    )
                    .accessible_name(text("mcp.save_mcp_settings"))
                    .clicked()
                {
                    event = Some(Event::Save(self.settings.clone()));
                }
            });
        event
    }
}

impl super::super::Probe {
    pub(super) fn save_mcp_settings(&mut self, settings: &str) {
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Workspace revision is unavailable".to_owned())
            .and_then(|snapshot| save_request(snapshot, settings))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                self.pending = Some(super::super::Pending::McpSave);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }
}

pub fn settings_text(snapshot: &Value) -> String {
    if has_no_mcp_servers(snapshot) {
        let sources = snapshot["config"]["goal_sources"].as_object().map_or_else(
            || "{}".to_owned(),
            |sources| serde_json::to_string(sources).expect("MCP goal sources are serializable"),
        );
        return EXAMPLE_SETTINGS.replace(
            "\"goal_sources\": {}",
            &format!("\"goal_sources\": {sources}"),
        );
    }
    let empty = Value::Object(serde_json::Map::new());
    let settings = json!({
        "mcp": snapshot["config"]["mcp"].as_object().map_or(&empty, |_| &snapshot["config"]["mcp"]),
        "goal_sources": snapshot["config"]["goal_sources"].as_object().map_or(&empty, |_| &snapshot["config"]["goal_sources"])
    });
    serde_json::to_string_pretty(&settings).expect("MCP settings JSON is serializable")
}

fn has_no_mcp_servers(snapshot: &Value) -> bool {
    snapshot["config"]["mcp"]
        .as_object()
        .is_none_or(|servers| servers.is_empty())
}

const EXAMPLE_SETTINGS: &str = r#"{
  "mcp": {
    // Local OpenCode server. Replace the command and uncomment these lines.
    // "company-local": {
    //   "type": "local",
    //   "command": ["your-mcp-server", "--stdio"],
    //   "environment": { "COMPANY_TOKEN": "{env:COMPANY_MCP_TOKEN}" },
    //   "enabled": true,
    //   "timeout": 5000
    // },
    // Remote OpenCode server. Replace the reserved example URL and uncomment.
    // "company-remote": {
    //   "type": "remote",
    //   "url": "https://mcp.example.invalid/mcp",
    //   "headers": { "Authorization": "Bearer {env:COMPANY_MCP_TOKEN}" },
    //   "enabled": true,
    //   "timeout": 5000
    // }
  },
  "goal_sources": {}
}"#;

pub fn save_request(snapshot: &Value, settings: &str) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .filter(|revision| !revision.is_empty())
        .ok_or_else(|| "Workspace revision is unavailable".to_owned())?;
    let packet = serde_json::to_vec(&json!({
        "kind": "mcp_save",
        "revision": revision,
        "settings": settings
    }))
    .map_err(|error| error.to_string())?;
    if packet.len() > 64 * 1024 {
        return Err(crate::ui::messages::text("native.native_request_exceeds_64_kib").into());
    }
    Ok(packet)
}
