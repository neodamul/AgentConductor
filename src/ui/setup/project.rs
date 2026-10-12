//! Select a Git root or a private snapshot of an ordinary project folder.
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

pub(super) fn setup_needed(path: &Path) -> Result<Option<&'static str>, String> {
    if path.as_os_str().is_empty() {
        return Err("choose a project folder before planning".into());
    }
    let path = directory(path)?;
    match Repository::open(&path) {
        Ok(repo) => Ok(repo.head().is_err().then_some("commit")),
        Err(_) if path.join("HEAD").is_file() && path.join("objects").is_dir() => {
            Err("choose a Git working directory rather than a bare repository".into())
        }
        Err(_) => Ok(Some("git")),
    }
}

impl Startup {
    pub(super) fn execution_repository(&self, path: &Path) -> Result<Repository, String> {
        if path.as_os_str().is_empty() {
            return Err("choose a project folder before planning".into());
        }
        crate::folder::repository(path, &self.broker.dir.join("folder-history"))
    }
}
