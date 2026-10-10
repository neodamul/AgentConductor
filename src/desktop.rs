//! Default native workspace selection (R-23 / DD-38).
use std::{ffi::OsString, path::PathBuf};

pub fn workspace() -> Result<PathBuf, String> {
    select_home(
        std::env::var_os("HOME"),
        std::env::var_os("USERPROFILE"),
        cfg!(windows),
    )
    .map(|home| home.join(".agentconductor/desktop"))
}

fn select_home(
    home: Option<OsString>,
    profile: Option<OsString>,
    windows: bool,
) -> Result<PathBuf, String> {
    home.filter(|value| !value.is_empty())
        .or_else(|| profile.filter(|value| windows && !value.is_empty()))
        .map(PathBuf::from)
        .ok_or_else(|| "Home directory is unavailable (HOME or Windows USERPROFILE); use ui-new <workspace-dir>".into())
}

#[cfg(test)]
#[path = "../tests/unit/desktop.rs"]
mod tests;
