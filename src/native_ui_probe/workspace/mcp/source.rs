use serde_json::{Value, json};
use std::collections::BTreeSet;

mod editor;
pub use editor::{Event, State};

pub fn append_reference(goal: &mut String, kind: &str, id: &str) -> Result<(), String> {
    if !["jira", "confluence"].contains(&kind) {
        return Err(crate::ui::messages::text("native.invalid_mcp_preview").into());
    }
    let reference = format!("{kind}:{id}");
    let parsed = crate::mcp::references(&reference)?;
    if parsed.len() != 1 || parsed[0].key() != reference {
        return Err(
            crate::ui::messages::text("mcp.enter_a_jira_issue_key_or_numeric_confluence").into(),
        );
    }
    if !crate::mcp::references(goal)?.contains(&parsed[0]) {
        if !goal.is_empty() && !goal.chars().last().is_some_and(char::is_whitespace) {
            goal.push(' ');
        }
        goal.push_str(&reference);
    }
    Ok(())
}

pub fn preview_request(snapshot: &Value) -> Result<Vec<u8>, String> {
    let revision = revision(snapshot)?;
    let goal = snapshot["config"]["goal"].as_str().unwrap_or_default();
    if crate::mcp::references(goal)?.is_empty() {
        return Err(crate::ui::messages::text(
            "native.add_explicit_jira_key_123_or_confluence_page",
        )
        .into());
    }
    request(json!({"kind":"mcp_preview","revision":revision}))
}

pub fn apply_request(
    snapshot: &Value,
    preview: &Value,
    selected: &[String],
) -> Result<Vec<u8>, String> {
    let revision = revision(snapshot)?;
    let id = preview["id"]
        .as_str()
        .filter(|id| !id.is_empty())
        .ok_or(crate::ui::messages::text("native.invalid_mcp_preview"))?;
    let records = preview["records"]
        .as_array()
        .filter(|records| !records.is_empty())
        .ok_or(crate::ui::messages::text("native.invalid_mcp_preview"))?;
    if preview["failures"].as_array().is_some_and(|failures| {
        failures
            .iter()
            .any(|failure| failure["reference"]["kind"] == "jira")
    }) {
        return Err(crate::ui::messages::text("native.mcp_preview_has_unresolved_jira").into());
    }
    let displayed = records
        .iter()
        .map(reference_key)
        .collect::<Result<BTreeSet<_>, _>>()?;
    if displayed.len() != records.len() || selected.is_empty() {
        return Err(
            crate::ui::messages::text("native.approve_the_exact_displayed_mcp_preview").into(),
        );
    }
    let selected_set = selected.iter().collect::<BTreeSet<_>>();
    if selected_set.len() != selected.len()
        || selected.iter().any(|key| !displayed.contains(key))
        || records.iter().any(|record| {
            record["reference"]["kind"] == "jira"
                && reference_key(record).is_ok_and(|key| !selected_set.contains(&key))
        })
    {
        return Err(
            crate::ui::messages::text("native.approve_the_exact_displayed_mcp_preview").into(),
        );
    }
    request(json!({"kind":"mcp_apply","revision":revision,"preview_id":id,"selected":selected}))
}

fn reference_key(record: &Value) -> Result<String, String> {
    let kind = record["reference"]["kind"].as_str().unwrap_or_default();
    let id = record["reference"]["id"].as_str().unwrap_or_default();
    let key = format!("{kind}:{id}");
    if crate::mcp::references(&key)?.len() != 1 {
        return Err(crate::ui::messages::text("native.invalid_mcp_preview").into());
    }
    Ok(key)
}

fn revision(snapshot: &Value) -> Result<&str, String> {
    snapshot["revision"]
        .as_str()
        .filter(|revision| !revision.is_empty())
        .ok_or_else(|| "Workspace revision is unavailable".to_owned())
}

fn request(value: Value) -> Result<Vec<u8>, String> {
    let packet = serde_json::to_vec(&value).map_err(|error| error.to_string())?;
    if packet.len() > 64 * 1024 {
        return Err(crate::ui::messages::text("native.native_request_exceeds_64_kib").into());
    }
    Ok(packet)
}

impl crate::native_ui_probe::Probe {
    pub(in crate::native_ui_probe) fn preview_mcp_sources(&mut self) {
        let snapshot = self.snapshot.clone();
        let result = snapshot
            .as_ref()
            .ok_or_else(|| "Workspace revision is unavailable".to_owned())
            .and_then(preview_request)
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                if let Some(snapshot) = snapshot.as_ref() {
                    self.mcp_sources.preview_started(snapshot);
                }
                self.pending = Some(super::super::Pending::McpPreview);
                self.workspace_next_refresh = None;
                self.error = None;
            }
            Err(error) => {
                self.mcp_sources.preview_failed(&error);
                self.error = Some(error);
            }
        }
    }

    pub(in crate::native_ui_probe) fn apply_mcp_sources(&mut self, selected: &[String]) {
        let packet = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Workspace revision is unavailable".to_owned())
            .and_then(|snapshot| {
                self.mcp_sources
                    .preview()
                    .ok_or_else(|| {
                        crate::ui::messages::text("native.invalid_mcp_preview").to_owned()
                    })
                    .and_then(|preview| apply_request(snapshot, preview, selected))
            });
        let result = packet.and_then(|packet| {
            self.worker
                .as_ref()
                .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                .send(&packet)
        });
        match result {
            Ok(()) => {
                self.mcp_sources.apply_started();
                self.pending = Some(super::super::Pending::McpApply);
                self.error = None;
            }
            Err(error) => {
                self.mcp_sources.preview_failed(&error);
                self.error = Some(error);
            }
        }
    }

    pub(in crate::native_ui_probe) fn refresh_mcp_imports(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"snapshot"}"#));
        match result {
            Ok(()) => self.pending = Some(super::super::Pending::McpApplyRefresh),
            Err(error) => self.handle_mcp_error(error),
        }
    }
}
