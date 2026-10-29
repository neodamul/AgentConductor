use serde_json::Value;

mod app;
pub mod appearance;
pub mod cache;
pub mod manual_viewer;
mod motion;
mod pending;
mod poll;
#[cfg(test)]
#[path = "../tests/native_ui_probe/poll_timeout.rs"]
mod poll_timeout_tests;
mod probe;
mod quota_refresh;
mod render;
pub mod room;
mod room_actions;
mod room_briefing;
pub mod updates;
mod worker;
#[cfg(test)]
#[path = "../tests/support/native_worker.rs"]
pub(crate) mod worker_test_support;
pub mod workspace;
pub use app::open_window;
use pending::Pending;
pub use render::{
    evidence as render_evidence, evidence_in_language as render_evidence_in_language, status_color,
    status_color_with_motion, status_colors, status_label,
};
pub use room::{CandidateDecision, candidate_decision_request, refresh_due, room_refresh_interval};
pub use room_actions::{RunAction, ToolAction, run_action_request, tool_action_request};
pub use worker::RequestWorker;
pub use workspace::{picked_folder, profile_from_template, workspace_save_request};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Surface {
    Workspace,
    AgentRoom,
}

#[cfg(feature = "manual-screenshot-capture")]
pub fn take_manual_screenshot_path(
    already_requested: bool,
    output_path: Option<std::ffi::OsString>,
    view_ready: bool,
) -> Option<std::ffi::OsString> {
    if already_requested || !view_ready {
        None
    } else {
        output_path
    }
}

#[derive(Default)]
pub struct Probe {
    #[cfg(feature = "manual-screenshot-capture")]
    manual_screenshot_requested: bool,
    #[cfg(feature = "manual-screenshot-capture")]
    manual_screenshot_settings_requested: bool,
    #[cfg(feature = "manual-screenshot-capture")]
    manual_screenshot_settings_ready_at: Option<std::time::Instant>,
    motion: motion::Policy,
    room_view: Option<room::State>,
    workspace_step: usize,
    workspace_next_refresh: Option<std::time::Instant>,
    quota_refresh_last: std::collections::BTreeMap<String, u64>,
    name: String,
    repository: String,
    goal: String,
    workspace_mode: crate::plan::WorkspaceMode,
    profiles: Vec<Value>,
    profile_editor: workspace::profile_tools::Editor,
    profile_saved: bool,
    executable_resolution: Option<String>,
    coordinator: String,
    roles: crate::roles::RoleRules,
    worker: Option<RequestWorker>,
    snapshot: Option<Value>,
    error: Option<String>,
    background_poll_failures: u8,
    pending: Option<Pending>,
    resume_after_tool_approval: bool,
    resume_after_task_feedback: bool,
    confirming_start: Option<(String, String)>,
    task_draft: Option<workspace::TaskDraft>,
    task_saved_notice: bool,
    plan_source: workspace::plan_source::State,
    starting_run: Option<String>,
    cache: cache::State,
    navigation: workspace::navigation::State,
    sharing: workspace::sharing::State,
    mcp_editor: workspace::mcp::State,
    mcp_sources: workspace::mcp::SourceImport,
    messenger_editor: workspace::messenger::State,
    signal_settings: workspace::signal_settings::State,
    coordinator_chat: workspace::chat::State,
    goal_optimizer: workspace::optimizer::State,
    updates: updates::State,
    update_source: updates::SourceEditor,
}
