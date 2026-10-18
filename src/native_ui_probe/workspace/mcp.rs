mod editor;
mod source;
pub use editor::{Event, State};
use serde_json::{Value, json};
pub use source::{
    Event as SourceEvent, State as SourceImport, append_reference,
    apply_request as source_apply_request, preview_request as source_preview_request,
};

impl super::super::Probe {
    pub(in crate::native_ui_probe) fn take_mcp_discovery(&mut self) {
        let Some(server) = self.mcp_editor.take_discovery() else {
            return;
        };
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Workspace revision is unavailable".to_owned())
            .and_then(|snapshot| discover_request(snapshot, &server))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                self.pending = Some(super::super::Pending::McpDiscover);
                self.workspace_next_refresh = None;
                self.error = None;
            }
            Err(error) => {
                self.mcp_editor.discovery_failed(&error);
                self.error = Some(error);
            }
        }
    }

    pub(in crate::native_ui_probe) fn refresh_mcp_snapshot(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"snapshot","mcp_result":true}"#));
        match result {
            Ok(()) => self.pending = Some(super::super::Pending::McpStatus),
            Err(error) => self.handle_mcp_error(error),
        }
    }

    pub(in crate::native_ui_probe) fn handle_mcp_error(&mut self, error: impl ToString) {
        let error = error.to_string();
        self.mcp_editor.discovery_failed(&error);
        self.mcp_sources.preview_failed(&error);
        self.workspace_next_refresh = None;
        self.pending = None;
        self.error = Some(error);
    }

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

pub fn discover_request(snapshot: &Value, server: &str) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .filter(|revision| !revision.is_empty())
        .ok_or_else(|| "Workspace revision is unavailable".to_owned())?;
    if snapshot["config"]["mcp"].get(server).is_none() {
        return Err(crate::ui::messages::text("native.unknown_mcp_server").into());
    }
    let packet = serde_json::to_vec(&json!({
        "kind": "mcp_discover",
        "revision": revision,
        "server": server
    }))
    .map_err(|error| error.to_string())?;
    if packet.len() > 64 * 1024 {
        return Err(crate::ui::messages::text("native.native_request_exceeds_64_kib").into());
    }
    Ok(packet)
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

fn server_names(snapshot: &Value) -> Vec<String> {
    snapshot["config"]["mcp"]
        .as_object()
        .map_or_else(Vec::new, |servers| servers.keys().cloned().collect())
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
