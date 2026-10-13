//! Private change history for projects without a committed Git repository.
//! Never initializes Git in the selected project; Git remains a runtime dependency.
use crate::{git::Repository, workflow::digest};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

const MAX_FILES: usize = 10_000;
const MAX_FILE: u64 = 16 * 1024 * 1024;
const MAX_TOTAL: u64 = 256 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileState {
    pub hash: String,
    pub executable: bool,
    pub bytes: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub original: PathBuf,
    pub workspace: PathBuf,
    pub base: String,
    pub files: BTreeMap<String, FileState>,
}

pub fn excluded(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    matches!(
        name.as_str(),
        ".git"
            | ".agentconductor"
            | "node_modules"
            | "target"
            | ".venv"
            | "venv"
            | "__pycache__"
            | ".ssh"
            | ".aws"
            | ".ds_store"
            | "thumbs.db"
            | "desktop.ini"
            | ".env"
    ) || name.starts_with(".env.")
        || name.starts_with("._")
}
pub(crate) fn ordinary(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return false;
        }
    }
    !metadata.file_type().is_symlink()
}
pub(super) fn state(path: &Path) -> Result<Option<FileState>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    if !ordinary(&metadata) || !metadata.is_file() {
        return Err(format!(
            "folder path is not a regular file: {}",
            path.display()
        ));
    }
    if metadata.len() > MAX_FILE {
        return Err("folder file exceeds 16 MiB; use a Git project for larger assets".into());
    }
    let mut data = Vec::new();
    crate::ui::open_artifact(path)?
        .take(MAX_FILE + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() as u64 > MAX_FILE {
        return Err("folder file grew beyond 16 MiB".into());
    }
    #[cfg(unix)]
    let executable = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    };
    #[cfg(not(unix))]
    let executable = false;
    Ok(Some(FileState {
        hash: digest(&data),
        executable,
        bytes: data.len() as u64,
    }))
}
pub fn scan(root: &Path) -> Result<BTreeMap<String, FileState>, String> {
    let metadata = fs::symlink_metadata(root).map_err(|e| e.to_string())?;
    if !ordinary(&metadata) || !metadata.is_dir() {
        return Err("original folder alias is unsupported".into());
    }
    fn walk(
        root: &Path,
        path: &Path,
        files: &mut BTreeMap<String, FileState>,
        total: &mut u64,
    ) -> Result<(), String> {
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| "folder paths must be UTF-8")?;
            if excluded(&name) {
                continue;
            }
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if !ordinary(&metadata) {
                return Err(format!(
                    "folder aliases are unsupported: {}",
                    path.display()
                ));
            }
            // A workspace's goal folder (marked by slugs::create_tree) is not project source.
            if metadata.is_dir() && path.join(crate::slugs::MARKER).is_file() {
                continue;
            }
            if metadata.is_dir() {
                walk(root, &path, files, total)?;
            } else {
                let value = state(&path)?.ok_or("folder file vanished during snapshot")?;
                *total += value.bytes;
                if files.len() >= MAX_FILES || *total > MAX_TOTAL {
                    return Err(
                        "folder snapshot exceeds 10,000 files or 256 MiB; use a Git project".into(),
                    );
                }
                let relative = path
                    .strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_str()
                    .ok_or("folder paths must be UTF-8")?
                    .replace('\\', "/");
                files.insert(relative, value);
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files, &mut 0)?;
    Ok(files)
}
pub fn prepare(original: &Path, parent: &Path) -> Result<PathBuf, String> {
    let original = original.canonicalize().map_err(|e| e.to_string())?;
    if !original.is_dir() {
        return Err("project path must be a directory".into());
    }
    if let Some(ancestor) = parent.ancestors().find(|p| p.is_dir())
        && ancestor
            .canonicalize()
            .map_err(|e| e.to_string())?
            .starts_with(&original)
    {
        return Err("private folder history must be outside the project".into());
    }
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let parent = parent.canonicalize().map_err(|e| e.to_string())?;
    if parent.starts_with(&original) {
        return Err("private folder history must be outside the project".into());
    }
    let files = scan(&original)?;
    let workspace = parent.join(format!("folder-{}", crate::native_guard::session_id()?));
    fs::create_dir(&workspace).map_err(|e| e.to_string())?;
    for path in files.keys() {
        let target = workspace.join(path);
        fs::create_dir_all(target.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::copy(original.join(path), target).map_err(|e| e.to_string())?;
    }
    let mut command = crate::subprocess::command("git");
    command
        .current_dir(&workspace)
        .args(["init", "--template=", "-b", "main"]);
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_COMMON_DIR",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    ] {
        command.env_remove(key);
    }
    let output = command
        .output()
        .map_err(|e| format!("private change history requires Git: {e}"))?;
    if !output.status.success() {
        return Err("cannot initialize private change history".into());
    }
    if scan(&workspace)? != files {
        return Err("folder changed while copying; retry planning".into());
    }
    let git = Repository::open(&workspace)?;
    git.command(&["add", "-f", "--", "."])?;
    git.command(&["commit", "--allow-empty", "-m", "Folder baseline"])?;
    let source = Source {
        original,
        workspace: workspace.clone(),
        base: git.head()?,
        files,
    };
    source.validate()?;
    fs::write(
        workspace.join(".git/agentconductor-folder.json"),
        serde_json::to_vec(&source).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(workspace)
}
pub fn load(workspace: &Path) -> Result<Option<Source>, String> {
    let path = workspace.join(".git/agentconductor-folder.json");
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read(path).map_err(|e| e.to_string())?;
    let source: Source = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
    if source.workspace != workspace.canonicalize().map_err(|e| e.to_string())? {
        return Err("folder workspace binding mismatch".into());
    }
    Ok(Some(source))
}
impl Source {
    pub fn validate(&self) -> Result<(), String> {
        if scan(&self.original)? != self.files {
            return Err(
                "original folder changed; generate a new plan before applying changes".into(),
            );
        }
        Ok(())
    }
    pub fn safe_path(&self, path: &str) -> Result<PathBuf, String> {
        let root = fs::symlink_metadata(&self.original).map_err(|e| e.to_string())?;
        if !ordinary(&root) || !root.is_dir() {
            return Err("folder apply refuses original-folder aliases".into());
        }
        let relative = Path::new(path);
        if relative.is_absolute()
            || path.contains('\\')
            || relative
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
            || path.split('/').any(excluded)
        {
            return Err("unsafe or excluded folder change path".into());
        }
        let mut current = self.original.clone();
        for component in relative.components() {
            current.push(component);
            if let Ok(metadata) = fs::symlink_metadata(&current)
                && !ordinary(&metadata)
            {
                return Err("folder apply refuses aliases".into());
            }
        }
        Ok(current)
    }
}
mod apply;

pub fn repository(path: &Path, history: &Path) -> Result<Repository, String> {
    if let Ok(repo) = Repository::open(path)
        && repo.head().is_ok()
    {
        return Ok(repo);
    }
    Repository::open(&prepare(path, history)?)
}

mod lifecycle;
