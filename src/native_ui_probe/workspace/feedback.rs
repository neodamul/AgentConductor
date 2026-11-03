use super::Probe;
use crate::native_ui_probe::Pending;
use eframe::egui;

#[cfg(test)]
#[path = "../../../tests/native_ui_probe/metadata_feedback.rs"]
mod tests;

pub(super) fn show(probe: &Probe, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if foreground_pending(probe) || probe.starting_run.is_some() {
            ui.spinner();
            ui.label(status(probe));
        }
        if let Some(error) = &probe.error {
            ui.colored_label(egui::Color32::DARK_RED, error);
        }
    });
}

pub(in crate::native_ui_probe) fn track_pending(probe: &mut Probe, now: std::time::Instant) {
    let kind = probe
        .pending
        .filter(|pending| request_wait_pending(*pending))
        .map(|pending| std::mem::discriminant(&pending));
    if kind.is_some() && kind != probe.pending_kind {
        probe.pending_since = Some(now);
    } else {
        if kind.is_none() {
            probe.pending_since = None;
        }
    }
    probe.pending_kind = kind;
}

pub(super) fn waiting_status(probe: &Probe) -> Option<String> {
    let since = probe.pending_since?;
    let elapsed = since.elapsed();
    if elapsed < std::time::Duration::from_secs(1)
        || !probe.pending.is_some_and(request_wait_pending)
    {
        return None;
    }
    Some(status(probe))
}

fn foreground_pending(probe: &Probe) -> bool {
    probe
        .pending
        .is_some_and(|pending| !matches!(pending, Pending::Metadata | Pending::MetadataRefresh))
}

fn request_wait_pending(pending: Pending) -> bool {
    !matches!(
        pending,
        Pending::BackgroundTaskDetails
            | Pending::BackgroundCoordinator
            | Pending::Refresh
            | Pending::PrepareRefresh
            | Pending::Metadata
            | Pending::MetadataRefresh
            | Pending::NavigationRefresh
            | Pending::StartRefresh
            | Pending::ProfileRefresh
            | Pending::ProfileRetryRefresh
            | Pending::CatalogRefresh
            | Pending::McpRefresh
            | Pending::McpApplyRefresh
            | Pending::MessengerRefresh
            | Pending::ChatRefresh
            | Pending::OptimizeRefresh
    )
}

pub(super) fn visible(probe: &Probe) -> bool {
    foreground_pending(probe) || probe.starting_run.is_some() || probe.error.is_some()
}

fn status(probe: &Probe) -> String {
    let activity = if probe.plan_source.is_picking_file() {
        probe.navigation.appearance.text("startup.read_plan")
    } else if probe.starting_run.is_some() {
        "Starting the run…"
    } else if let Some(id) = match probe.pending {
        Some(Pending::MessengerLoad) => Some("startup.loading_messenger_settings"),
        Some(Pending::MessengerTest) => Some("startup.sending_test_message"),
        Some(Pending::MessengerSave | Pending::MessengerRefresh) => {
            Some("startup.saving_messenger_settings")
        }
        _ => None,
    } {
        probe.navigation.appearance.text(id)
    } else {
        match probe.pending {
            Some(Pending::PickFolder) => "Choosing project folder…",
            Some(Pending::NewGoal) => "Starting a new goal…",
            Some(Pending::Prepare) => "Generating a plan…",
            Some(Pending::NextPhase) => "Updating the plan…",
            Some(Pending::Metadata | Pending::MetadataRefresh) => "Refreshing agent metadata…",
            Some(Pending::Start) => "Starting the run…",
            Some(Pending::Refresh)
                if probe
                    .snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot["job"]["state"] == "running") =>
            {
                "Refreshing plan progress…"
            }
            Some(Pending::Navigation | Pending::NavigationRefresh | Pending::SelectGoal) => {
                "Updating workspaces…"
            }
            Some(Pending::Coordinator) => "Loading coordinator status…",
            Some(Pending::Briefing) => "Preparing team briefing…",
            Some(Pending::TaskDetails) => "Loading task details…",
            Some(Pending::Action(description)) => description,
            Some(Pending::ToolDecision { description, .. }) => description,
            Some(Pending::Evidence) => "Collecting review evidence…",
            Some(Pending::ScopeFailure | Pending::ScopeRecoveryPreview) => {
                "Checking recovery options…"
            }
            Some(Pending::ScopeRecoveryApply) => "Applying recovery…",
            Some(Pending::StartRefresh | Pending::OpenRun) => "Opening the run…",
            Some(Pending::CachePreview) => "Previewing cache cleanup…",
            Some(Pending::CacheClear) => "Cleaning cached files…",
            Some(
                Pending::ShareFolder(_)
                | Pending::ShareExport
                | Pending::SharePreview
                | Pending::ShareImport,
            ) => "Updating workspace sharing…",
            Some(
                Pending::McpDiscover
                | Pending::McpPreview
                | Pending::McpApply
                | Pending::McpApplyRefresh
                | Pending::McpStatus
                | Pending::McpRefresh,
            ) => "Updating MCP servers…",
            Some(Pending::MessengerLoad) => "Loading messenger settings…",
            Some(Pending::MessengerTest) => "Sending a test message…",
            Some(Pending::MessengerSave | Pending::MessengerRefresh) => {
                "Saving messenger settings…"
            }
            Some(Pending::SignalSettingsLoad | Pending::SignalSettingsSave) => {
                "Updating signal settings…"
            }
            Some(Pending::UpdateSourceLoad | Pending::UpdateSourceSave) => {
                "Updating release source…"
            }
            Some(Pending::Update) => "Checking for updates…",
            Some(Pending::ChatStart) => "Sending coordinator message…",
            Some(Pending::ChatReset) => "Starting a new conversation…",
            Some(Pending::OptimizeStart | Pending::OptimizeRefresh) => "Optimizing the goal…",
            Some(Pending::ProfileSave | Pending::ProfileRefresh | Pending::ProfileRetryRefresh) => {
                "Saving agent settings…"
            }
            Some(Pending::CatalogSave | Pending::CatalogRefresh) => "Updating agent catalog…",
            Some(Pending::PickExecutable | Pending::ResolveExecutable) => {
                "Selecting an executable…"
            }
            _ => "Saving workspace…",
        }
    };
    let Some(since) = probe.pending_since else {
        return activity.to_owned();
    };
    let elapsed = since.elapsed();
    if elapsed < std::time::Duration::from_secs(1) {
        return activity.to_owned();
    }
    let elapsed = format!("{}:{:02}", elapsed.as_secs() / 60, elapsed.as_secs() % 60);
    crate::ui::messages::interpolate(
        probe.navigation.appearance.text("startup.waiting_for_app"),
        &[("value1", activity), ("value2", &elapsed)],
    )
}
