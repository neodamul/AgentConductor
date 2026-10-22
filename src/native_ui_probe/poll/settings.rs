use serde_json::Value;

use super::super::{Pending, Probe, workspace};

pub(super) fn apply(probe: &mut Probe, snapshot: Value) {
    match probe.pending {
        Some(Pending::PickFolder) => {
            if probe.plan_source.is_picking_file() {
                workspace::plan_source::picked_file(probe, &snapshot);
            } else {
                probe.repository = workspace::picked_folder(&probe.repository, &snapshot);
                probe.pending = None;
                probe.error = None;
            }
        }
        Some(Pending::ShareFolder(action)) => probe.share_folder_reply(snapshot, action),
        Some(Pending::ShareExport) => {
            probe.sharing.exported(&snapshot);
            probe.pending = None;
            probe.error = None;
        }
        Some(Pending::SharePreview) => {
            if snapshot["settings"].is_object() {
                probe.sharing.set_preview(snapshot);
                probe.pending = None;
                probe.error = None;
            } else {
                probe.pending = None;
                probe.error = Some("Workspace settings preview was not returned".into());
            }
        }
        Some(Pending::ShareImport) if snapshot["imported"] == true => {
            probe.sharing.imported();
            probe.refresh_after(Pending::Refresh);
        }
        Some(Pending::ShareImport) => {
            probe.pending = None;
            probe.error = Some("Workspace settings were not imported".into());
        }
        Some(Pending::PickExecutable) => {
            if let Some(path) = snapshot["picked_executable"].as_str() {
                probe.profile_editor.set_executable(path);
            }
            probe.pending = None;
        }
        Some(Pending::ResolveExecutable) => {
            if let Some(requested) = probe.executable_resolution.take() {
                probe
                    .profile_editor
                    .apply_resolution(requested.as_str(), snapshot["path"].as_str());
            }
            probe.pending = None;
        }
        Some(Pending::ProfileSave) if snapshot["saved"] == true => {
            probe.refresh_after(Pending::ProfileRefresh)
        }
        Some(Pending::CatalogSave) if snapshot["saved"] == true => {
            probe.refresh_after(Pending::CatalogRefresh)
        }
        Some(Pending::CatalogSave) => {
            probe.pending = None;
            probe.profile_editor.set_catalog_error(Some(
                snapshot["error"]
                    .as_str()
                    .unwrap_or("Model catalog was not saved")
                    .to_owned(),
            ));
        }
        Some(Pending::CatalogRefresh) => {
            probe.profiles = snapshot["config"]["profiles"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            probe.snapshot = Some(snapshot);
            probe.profile_editor.close_catalog();
            probe.pending = None;
            probe.error = None;
        }
        Some(Pending::ProfileSave) => {
            probe.pending = None;
            let error = snapshot["error"]
                .as_str()
                .unwrap_or("The saved profile failed validation");
            if probe.profile_editor.is_editing() {
                probe
                    .profile_editor
                    .set_save_error(probe.navigation.appearance.language(), error);
                probe.refresh_after(Pending::ProfileRetryRefresh);
            } else {
                probe.error = Some(error.to_owned());
            }
        }
        Some(Pending::ProfileRefresh) => {
            let renames = probe.profile_editor.renames().clone();
            workspace::profile_tools::rename_references(
                &mut probe.profiles,
                &mut probe.coordinator,
                &mut probe.roles,
                &renames,
            );
            if probe.coordinator.is_empty() {
                probe.coordinator = snapshot["config"]["coordinator"]
                    .as_str()
                    .unwrap_or_default()
                    .into();
            }
            probe.profile_editor.finish_save();
            probe.profile_saved = true;
            probe.profiles = snapshot["config"]["profiles"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            probe.snapshot = Some(snapshot);
            probe.pending = None;
            probe.error = None;
        }
        Some(Pending::ProfileRetryRefresh) => {
            probe.profiles = snapshot["config"]["profiles"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            probe.snapshot = Some(snapshot);
            probe.pending = None;
        }
        Some(Pending::McpSave) if snapshot["saved"] == true => {
            probe.refresh_after(Pending::McpRefresh)
        }
        Some(Pending::McpSave) => {
            probe.pending = None;
            probe.error = Some("MCP settings were not saved".into());
        }
        Some(Pending::McpRefresh) => {
            probe.snapshot = Some(snapshot.clone());
            probe.mcp_editor.saved();
            probe.mcp_sources.clear_preview();
            probe.pending = None;
            probe.error = None;
        }
        Some(Pending::MessengerLoad) => {
            probe.messenger_editor.apply_status(&snapshot);
            probe.pending = None;
            probe.error = None;
        }
        Some(Pending::MessengerTest) if snapshot["sent"] == true => {
            probe
                .messenger_editor
                .test_sent(probe.navigation.appearance.language());
            probe.pending = None;
            probe.error = None;
        }
        Some(Pending::MessengerTest) => {
            probe.pending = None;
            probe.error = Some(
                probe
                    .navigation
                    .appearance
                    .text("startup.messenger_test_failed")
                    .into(),
            );
        }
        Some(Pending::MessengerSave) if snapshot["settings"].is_object() => {
            probe.messenger_editor.apply_status(&snapshot);
            probe.refresh_after(Pending::MessengerRefresh);
        }
        Some(Pending::MessengerSave) => {
            probe.pending = None;
            probe.error = Some(
                probe
                    .navigation
                    .appearance
                    .text("startup.messenger_save_failed")
                    .into(),
            );
        }
        Some(Pending::MessengerRefresh) => {
            probe.snapshot = Some(snapshot);
            probe.messenger_editor.saved();
            probe.pending = None;
            probe.error = None;
        }
        Some(Pending::UpdateSourceLoad) if snapshot["default_repository"].is_string() => {
            probe.update_source.apply_settings(&snapshot);
            probe.pending = None;
            probe.error = None;
        }
        Some(Pending::UpdateSourceLoad) => {
            probe.pending = None;
            probe.update_source.set_error(
                probe
                    .navigation
                    .appearance
                    .text("startup.update_repository_load_failed")
                    .into(),
            );
        }
        Some(Pending::UpdateSourceSave) if snapshot["repository"].is_string() => {
            probe.update_source.saved();
            probe.pending = None;
            probe.error = None;
        }
        Some(Pending::UpdateSourceSave) => {
            probe.pending = None;
            probe.update_source.set_error(
                probe
                    .navigation
                    .appearance
                    .text("startup.update_repository_save_failed")
                    .into(),
            );
        }
        Some(Pending::Update) => {
            probe.updates.apply_status(snapshot);
            probe.pending = None;
            probe.error = None;
        }
        Some(Pending::Save) if snapshot["saved"] == true => probe.refresh_after(Pending::Refresh),
        Some(Pending::Save) => {
            workspace::plan_source::cancel(probe);
            probe.pending = None;
            probe.error = Some("Workspace goal was not saved".into());
        }
        Some(Pending::Prepare) => {
            workspace::plan_source::accepted(probe);
            probe.refresh_after(Pending::PrepareRefresh);
        }
        Some(Pending::NextPhase) => {
            probe.workspace_step = 2;
            probe.refresh_after(Pending::Refresh);
        }
        Some(Pending::SelectGoal) if snapshot["changed"] == true => {
            probe.refresh_after(Pending::Refresh);
        }
        Some(Pending::SelectGoal) => {
            probe.pending = None;
            probe.error = Some(
                probe
                    .navigation
                    .appearance
                    .text("startup.goal_selection_failed")
                    .into(),
            );
        }
        Some(Pending::Metadata) => probe.refresh_after(Pending::MetadataRefresh),
        Some(Pending::MetadataRefresh) => {
            if let Some(saved) = snapshot["config"]["profiles"].as_array() {
                workspace::profile_tools::merge_catalogs(&mut probe.profiles, saved);
            }
            probe.snapshot = Some(snapshot);
            probe.pending = None;
            probe.error = None;
        }
        Some(Pending::ChatStart) => probe.chat_started(snapshot),
        Some(Pending::ChatRefresh) => probe.chat_refreshed(snapshot),
        _ => {}
    }
}
