//! DD-133: selected-folder execution with all Git metadata outside the project.
use super::*;

pub fn history_directory(project: &Path) -> Result<PathBuf, String> {
    let project = project.canonicalize().map_err(|e| e.to_string())?;
    let name = crate::git::git_path(project.as_os_str())
        .to_str()
        .ok_or("non-UTF8 project path")?
        .to_string();
    let key = if cfg!(windows) {
        name.to_lowercase()
    } else {
        name
    };
    Ok(crate::desktop::workspace()?
        .parent()
        .ok_or("missing application history root")?
        .join("project-history")
        .join(digest(key.as_bytes())))
}

pub(crate) fn external_metadata(project: &Path) -> Result<Option<PathBuf>, String> {
    let history = history_directory(project)?;
    let binding = history.join("binding.json");
    if !binding.exists() {
        return Ok(None);
    }
    let mut data = Vec::new();
    crate::ui::open_artifact(&binding)?
        .take(8193)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > 8192 {
        return Err(crate::ui::messages::text("folder.workspace_binding_changed").into());
    }
    let value: serde_json::Value = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
    let selected = project.canonicalize().map_err(|e| e.to_string())?;
    if value["schema"] != 1 || value["project"] != serde_json::json!(selected) {
        return Err(crate::ui::messages::text("folder.workspace_binding_changed").into());
    }
    let metadata = history.join(".git");
    let info = fs::symlink_metadata(&metadata).map_err(|e| e.to_string())?;
    if !ordinary(&info)
        || !info.is_dir()
        || metadata
            .canonicalize()
            .map_err(|e| e.to_string())?
            .starts_with(&selected)
    {
        return Err(crate::ui::messages::text("folder.workspace_binding_changed").into());
    }
    Ok(Some(metadata))
}

fn exclusions(project: &Path) -> Result<String, String> {
    let mut text = String::from(
        "# AgentConductor local runtime exclusions\n.agentconductor/\nnode_modules/\ntarget/\n.venv/\nvenv/\n__pycache__/\n.ssh/\n.aws/\n.env\n.env.*\n.DS_Store\nThumbs.db\ndesktop.ini\n._*\n",
    );
    for entry in fs::read_dir(project).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "project paths must be UTF-8")?;
        if excluded(&name) || entry.path().join(crate::slugs::MARKER).is_file() {
            let escaped = name
                .chars()
                .flat_map(|c| {
                    if matches!(c, '*' | '?' | '[' | ']' | '\\') {
                        vec!['\\', c]
                    } else {
                        vec![c]
                    }
                })
                .collect::<String>();
            text.push_str(&format!("/{escaped}\n"));
        }
    }
    Ok(text)
}

pub fn activate(repo: &Repository, selected: &Path) -> Result<Repository, String> {
    let selected = selected.canonicalize().map_err(|e| e.to_string())?;
    let Some(source) = load(&repo.root)? else {
        if repo.root != selected {
            return Err(crate::ui::messages::text("folder.workspace_binding_changed").into());
        }
        return Ok(repo.clone());
    };
    if source.original != selected || repo.head()? != source.base {
        return Err(crate::ui::messages::text("folder.workspace_binding_changed").into());
    }
    let _lease = crate::leases::Lease::repository(&selected)?;
    source.validate()?;
    repo.clean()?;
    if fs::symlink_metadata(selected.join(".git")).is_ok() {
        if Repository::open(&selected).is_ok_and(|original| original.head().is_err()) {
            return Ok(repo.clone());
        }
        return Err(crate::ui::messages::text("folder.existing_history_requires_plan").into());
    }
    if let Ok(existing) = Repository::open(&selected) {
        if existing.is_external() && existing.head()? == source.base {
            return Ok(existing);
        }
        return Err(crate::ui::messages::text("folder.workspace_binding_changed").into());
    }
    let history = history_directory(&selected)?;
    if let Some(ancestor) = history.ancestors().find(|path| path.is_dir())
        && ancestor
            .canonicalize()
            .map_err(|e| e.to_string())?
            .starts_with(&selected)
    {
        return Err(crate::ui::messages::text(
            "native.startup_artifacts_must_be_outside_repository",
        )
        .into());
    }
    if history.starts_with(&selected) {
        return Err(crate::ui::messages::text(
            "native.startup_artifacts_must_be_outside_repository",
        )
        .into());
    }
    fs::create_dir_all(history.parent().ok_or("missing history parent")?)
        .map_err(|e| e.to_string())?;
    let staging = history.with_extension(format!("{}.pending", crate::native_guard::session_id()?));
    fs::create_dir(&staging).map_err(|e| e.to_string())?;
    let _stage = super::lifecycle::Stage(staging.clone());
    repo.command(&[
        "clone",
        "--no-checkout",
        "--no-hardlinks",
        "--",
        repo.root.to_str().ok_or("non-UTF8 history path")?,
        staging.to_str().ok_or("non-UTF8 history path")?,
    ])?;
    let local = Repository::open(&staging)?;
    local.command(&["remote", "remove", "origin"])?;
    local.command(&["read-tree", &source.base])?;
    local.command(&[
        "config",
        "core.worktree",
        selected.to_str().ok_or("non-UTF8 project path")?,
    ])?;
    fs::write(staging.join(".git/info/exclude"), exclusions(&selected)?)
        .map_err(|e| e.to_string())?;
    fs::write(
        staging.join("binding.json"),
        serde_json::to_vec(&serde_json::json!({"project":selected,"schema":1}))
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::OpenOptions::new()
        .write(true)
        .open(staging.join("binding.json"))
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())?;
    source.validate()?;
    if fs::symlink_metadata(&history).is_ok() {
        return Err(crate::ui::messages::text("folder.workspace_binding_changed").into());
    }
    fs::rename(&staging, &history).map_err(|e| e.to_string())?;
    let activated = Repository::open(&selected)?;
    if activated.head()? != source.base {
        return Err(crate::ui::messages::text("folder.workspace_binding_changed").into());
    }
    activated.clean()?;
    Ok(activated)
}

pub(super) fn baseline(repo: Repository, history: &Path) -> Result<Repository, String> {
    if !repo.is_external() {
        return Ok(repo);
    }
    let _lease = crate::leases::WorkspaceLease::acquire(&repo.root, &history.join("planning"))?;
    scan(&repo.root)?;
    let metadata = external_metadata(&repo.root)?.ok_or("external history unavailable")?;
    fs::write(metadata.join("info/exclude"), exclusions(&repo.root)?).map_err(|e| e.to_string())?;
    if !repo.changed(&repo.head()?)?.is_empty() {
        let base = repo.head()?;
        let snapshot = repo.checkpoint(
            &base,
            &metadata.join(format!(
                "baseline-{}.index",
                crate::native_guard::session_id()?
            )),
        )?;
        repo.merge_shared(&repo.branch()?, &base, &snapshot.commit)?;
    }
    repo.clean()?;
    Ok(repo)
}

pub fn working_repository(
    path: &Path,
    history: &Path,
    expected_base: Option<&str>,
) -> Result<Repository, String> {
    if let Ok(repo) = Repository::open(path)
        && repo.head().is_ok()
    {
        return if expected_base.is_none() {
            baseline(repo, history)
        } else {
            Ok(repo)
        };
    }
    let repo = Repository::open(&prepare(path, history)?)?;
    if expected_base.is_some_and(|base| repo.head().as_deref() != Ok(base)) {
        return Err("repository base changed since plan approval".into());
    }
    activate(&repo, path)
}
