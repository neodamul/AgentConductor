//! Startup requests accepted from the native page.
use super::*;

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Request {
    StopPlanning {},
    CachePreview {},
    CacheClear {
        digest: String,
    },
    McpSave {
        revision: String,
        settings: String,
    },
    McpDiscover {
        revision: String,
        server: String,
    },
    McpPreview {
        revision: String,
    },
    Chat {
        revision: String,
        text: String,
    },
    ChatReset {
        revision: String,
    },
    OptimizeGoal {
        revision: String,
        goal: String,
    },
    McpApply {
        revision: String,
        preview_id: String,
        selected: Vec<String>,
    },
    Snapshot {
        #[serde(default)]
        quota_refresh_last: std::collections::BTreeMap<String, u64>,
        #[serde(default)]
        mcp_result: bool,
        #[serde(default)]
        chat_result: bool,
        #[serde(default)]
        chat_history: bool,
        #[serde(default)]
        optimize_result: bool,
        /// DD-71: icon data URLs only when the page asks for them.
        #[serde(default)]
        icons: bool,
    },
    Save {
        revision: String,
        /// Boxed: the workspace configuration dwarfs every other request.
        config: Box<Config>,
    },
    Quota {
        revision: String,
        agent: String,
    },
    Discover {
        revision: String,
        agent: String,
    },
    Catalog {
        revision: String,
        agent: String,
        catalog: ModelCatalog,
    },
    Prepare {
        revision: String,
        /// DD-61: slug for a new goal; suggested from the goal when absent.
        #[serde(default)]
        slug: Option<String>,
    },
    NextPhase {
        revision: String,
    },
    SavePlanRules {
        revision: String,
        scope: String,
        #[serde(default)]
        text: Option<String>,
    },
    Draft {
        revision: String,
    },
    ReadPlan {
        revision: String,
        path: PathBuf,
        #[serde(default)]
        slug: Option<String>,
    },
    BaselineStatus {},
    CommitBaseline {
        revision: String,
    },
    IgnoreOsFiles {
        revision: String,
    },
    NewGoal {
        revision: String,
    },
    SelectGoal {
        revision: String,
        slug: String,
    },
    CreateSlug {
        revision: String,
        #[serde(default)]
        slug: Option<String>,
    },
    SaveProfiles {
        revision: String,
        profiles: Vec<AgentProfile>,
        /// DD-74: old ID → new ID for agents renamed in this save.
        #[serde(default)]
        renames: std::collections::BTreeMap<String, String>,
    },
    ExportSettings {
        revision: String,
        folder: PathBuf,
    },
    PreviewSettings {
        folder: PathBuf,
    },
    ImportSettings {
        revision: String,
        folder: PathBuf,
        keep_verified: bool,
    },
    Edit {
        revision: String,
        #[serde(default)]
        digest: Option<String>,
        tasks: Vec<Task>,
        workspace_mode: WorkspaceMode,
    },
    Start {
        revision: String,
        digest: String,
    },
    /// DD-79: messenger settings and whether a URL is saved (the URL is never returned).
    Messenger {},
    TestMessenger {
        settings: crate::messenger::Settings,
        #[serde(default)]
        url: Option<String>,
    },
    SaveMessenger {
        revision: String,
        settings: crate::messenger::Settings,
        /// `None` keeps the saved URL; `Some("")` removes it.
        #[serde(default)]
        url: Option<String>,
        /// DD-81 secrets, same convention: Slack bot token, relay URL and relay secret.
        #[serde(default)]
        token: Option<String>,
        #[serde(default)]
        relay_url: Option<String>,
        #[serde(default)]
        relay_secret: Option<String>,
    },
}
