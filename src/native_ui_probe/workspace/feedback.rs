use super::{Probe, pending_message};
use crate::native_ui_probe::Pending;
use eframe::egui;

pub(super) fn show(probe: &Probe, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if probe.pending.is_some() || probe.starting_run.is_some() {
            ui.spinner();
            ui.label(status(probe));
        }
        if let Some(error) = &probe.error {
            ui.colored_label(egui::Color32::DARK_RED, error);
        }
    });
}

fn status(probe: &Probe) -> String {
    if probe.plan_source.is_picking_file() {
        return probe.navigation.appearance.text("startup.read_plan").into();
    }
    if probe.starting_run.is_some() {
        return pending_message("start").into();
    }
    if let Some(id) = match probe.pending {
        Some(Pending::MessengerLoad) => Some("startup.loading_messenger_settings"),
        Some(Pending::MessengerTest) => Some("startup.sending_test_message"),
        Some(Pending::MessengerSave | Pending::MessengerRefresh) => {
            Some("startup.saving_messenger_settings")
        }
        _ => None,
    } {
        return probe.navigation.appearance.text(id).into();
    }
    let activity = match probe.pending {
        Some(Pending::PickFolder) => "folder",
        Some(Pending::Prepare | Pending::NextPhase) => "prepare",
        Some(Pending::Metadata) => "metadata",
        Some(Pending::MetadataRefresh) => "metadata_refresh",
        Some(Pending::Start) => "start",
        Some(Pending::Refresh)
            if probe
                .snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot["job"]["state"] == "running") =>
        {
            "plan_refresh"
        }
        Some(Pending::Navigation | Pending::NavigationRefresh | Pending::SelectGoal) => "workspace",
        Some(Pending::McpSave) => {
            return probe
                .navigation
                .appearance
                .text("mcp.saving_mcp_settings")
                .into();
        }
        _ => "save",
    };
    pending_message(activity).into()
}
