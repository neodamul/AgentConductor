use super::super::{Pending, Probe};
use serde_json::Value;

pub(super) fn apply(probe: &mut Probe, snapshot: Value) {
    match probe.pending {
        Some(Pending::McpDiscover) => probe.refresh_mcp_snapshot(),
        Some(Pending::McpStatus) => {
            let job = &snapshot["job"];
            probe.snapshot = Some(snapshot.clone());
            match (job["kind"].as_str(), job["state"].as_str()) {
                (Some("mcp"), Some("running")) => {
                    probe.pending = None;
                    probe.error = None;
                    probe.workspace_next_refresh =
                        Some(std::time::Instant::now() + std::time::Duration::from_secs(1));
                }
                (Some("mcp"), Some("completed")) if snapshot["mcp_result"]["tools"].is_array() => {
                    probe.mcp_editor.discovery_result(&snapshot["mcp_result"]);
                    probe.pending = None;
                    probe.error = None;
                    probe.workspace_next_refresh = None;
                }
                (Some("mcp"), Some("failed" | "interrupted" | "stopped")) => {
                    let message = job["error"].as_str().unwrap_or("MCP discovery failed");
                    probe.handle_mcp_error(message);
                }
                _ => probe.handle_mcp_error(
                    probe
                        .navigation
                        .appearance
                        .text("native.invalid_mcp_result"),
                ),
            }
        }
        _ => {}
    }
}
