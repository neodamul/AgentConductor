//! Select a Git root or a private snapshot of an ordinary project folder.
use super::*;
pub(super) fn directory(path: &Path) -> Result<PathBuf, String> {
    let path = path
        .canonicalize()
        .map_err(|_| crate::ui::messages::text("native.project_directory_does_not_exist"))?;
    if !path.is_dir() {
        return Err(crate::ui::messages::text("native.project_path_must_be_a_directory").into());
    }
    Ok(PathBuf::from(crate::git::git_path(path.as_os_str())))
}
pub(super) fn repository(path: &Path) -> Result<Repository, String> {
    if path.as_os_str().is_empty() {
        return Err(
            crate::ui::messages::text("native.choose_a_project_folder_before_planning").into(),
        );
    }
    let repo = Repository::open(path).map_err(|_| {
        crate::ui::messages::text("native.choose_the_root_of_a_git_repository_initialize")
            .to_string()
    })?;
    repo.head().map_err(|_| {
        crate::ui::messages::text("native.create_an_initial_commit_in_the_git_repository")
            .to_string()
    })?;
    Ok(repo)
}

pub(super) fn setup_needed(path: &Path) -> Result<Option<&'static str>, String> {
    if path.as_os_str().is_empty() {
        return Err(
            crate::ui::messages::text("native.choose_a_project_folder_before_planning").into(),
        );
    }
    let path = directory(path)?;
    match Repository::open(&path) {
        Ok(repo) => Ok(repo.head().is_err().then_some("commit")),
        Err(_) if path.join("HEAD").is_file() && path.join("objects").is_dir() => Err(
            crate::ui::messages::text("native.choose_a_git_working_directory_rather_than_a").into(),
        ),
        Err(_) => Ok(Some("git")),
    }
}

impl Startup {
    pub(super) fn execution_repository(&self, saved: &Saved) -> Result<Repository, String> {
        let path = &saved.config.repository;
        self.prune_history(saved)?;
        if path.as_os_str().is_empty() {
            return Err(crate::ui::messages::text(
                "native.choose_a_project_folder_before_planning",
            )
            .into());
        }
        crate::folder::repository(path, &self.broker.dir.join("folder-history"))
    }
}
