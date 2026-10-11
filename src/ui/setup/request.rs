//! Startup requests accepted from the native page.
use super::*;

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Request {
    Snapshot {
        /// DD-71: icon data URLs only when the page asks for them.
        #[serde(default)]
        icons: bool,
    },
    Save {
        revision: String,
        config: Config,
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
        tasks: Vec<Task>,
        workspace_mode: WorkspaceMode,
    },
    Start {
        revision: String,
        digest: String,
    },
    /// DD-78: messenger settings and whether a URL is saved (the URL is never returned).
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
    },
}
