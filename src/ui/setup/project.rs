//! Settings accept a project directory; planning requires a committed Git baseline.
use super::*;
pub(super) fn directory(path: &Path) -> Result<PathBuf, String> {
    let path = path
        .canonicalize()
        .map_err(|_| "project directory does not exist")?;
    if !path.is_dir() {
        return Err("project path must be a directory".into());
    }
    Ok(path)
}
pub(super) fn repository(path: &Path) -> Result<Repository, String> {
    if path.as_os_str().is_empty() {
        return Err("choose a project folder before planning".into());
    }
    let repo = Repository::open(path).map_err(|_| {
        "choose the root of a Git repository; initialize Git before planning".to_string()
    })?;
    repo.head().map_err(|_| {
        "create an initial commit in the Git repository before planning".to_string()
    })?;
    Ok(repo)
}
