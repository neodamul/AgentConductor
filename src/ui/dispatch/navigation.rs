use crate::cooperative::route::Preference;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Request {
    OpenRun {},
    Home {},
    PickFolder {},
    PickFile {},
    PickExecutable {},
    ResolveExecutable {
        executable: String,
    },
    Workspaces {},
    SwitchWorkspace {
        path: PathBuf,
    },
    CreateWorkspace {
        name: String,
    },
    ForgetWorkspace {
        path: PathBuf,
    },
    DeleteWorkspace {
        path: PathBuf,
        name: String,
        replacement: Option<String>,
    },
    SavedWorkspaces {},
    OpenWorkspace {
        path: PathBuf,
    },
    SetLanguage {
        language: String,
    },
    SetTheme {
        theme: String,
    },
    SignalTransport {},
    SetSignalTransport {
        preference: Preference,
    },
}
