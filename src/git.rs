use crate::plan::overlaps;
use serde::{Deserialize, Serialize};
#[path = "git/snapshot.rs"]
mod snapshot;

use std::{
    collections::BTreeSet,
    io::Write,
    path::{Path, PathBuf},
    process::Stdio,
};

#[derive(Clone)]
pub struct Repository {
    pub root: PathBuf,
    history: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub commit: String,
    pub tree: String,
    pub changed: Vec<String>,
}

impl Repository {
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut repo = Self {
            root: path.canonicalize().map_err(|e| e.to_string())?,
            history: None,
        };
        let actual = repo.text(&["rev-parse", "--show-toplevel"]);
        if actual.as_ref().is_ok_and(|p| {
            Path::new(p)
                .canonicalize()
                .is_ok_and(|root| root == repo.root)
        }) {
            return Ok(repo);
        }
        if std::fs::symlink_metadata(repo.root.join(".git")).is_ok() {
            return Err(actual
                .err()
                .unwrap_or_else(|| "repository path must be its root".into()));
        }
        repo.history = crate::folder::external_metadata(&repo.root)?;
        if repo.history.is_none() {
            return Err(actual
                .err()
                .unwrap_or_else(|| "repository path must be its root".into()));
        }
        let actual = repo.text(&["rev-parse", "--show-toplevel"])?;
        if Path::new(&actual)
            .canonicalize()
            .map_err(|e| e.to_string())?
            != repo.root
        {
            return Err("repository path must be its root".into());
        }
        Ok(repo)
    }
    pub fn is_external(&self) -> bool {
        self.history.is_some()
    }
    /// For a project without its own `.git`: the Git environment that points `git` at the
    /// history kept outside it, so commands such as `git diff --check` still work there.
    pub fn external_environment(&self) -> std::collections::BTreeMap<String, String> {
        let mut environment = std::collections::BTreeMap::new();
        if let Some(history) = &self.history {
            environment.insert(
                "GIT_DIR".into(),
                git_path(history.as_os_str()).to_string_lossy().into_owned(),
            );
            environment.insert(
                "GIT_WORK_TREE".into(),
                git_path(self.root.as_os_str())
                    .to_string_lossy()
                    .into_owned(),
            );
        }
        environment
    }
    fn invoke(
        &self,
        args: &[&str],
        input: Option<&[u8]>,
        index: Option<&Path>,
    ) -> Result<Vec<u8>, String> {
        let mut command = crate::subprocess::command("git");
        if let Some(history) = &self.history {
            command
                .arg("--git-dir")
                .arg(git_path(history.as_os_str()))
                .arg("--work-tree")
                .arg(git_path(self.root.as_os_str()));
        }
        command
            .arg("-C")
            .arg(git_path(self.root.as_os_str()))
            .args([
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.fsmonitor=false",
                "-c",
                "core.longpaths=true",
            ])
            .args(args.iter().map(|a| git_path(std::ffi::OsStr::new(a))))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("GIT_AUTHOR_NAME", "AgentConductor")
            .env("GIT_AUTHOR_EMAIL", "agentconductor@localhost")
            .env("GIT_COMMITTER_NAME", "AgentConductor")
            .env("GIT_COMMITTER_EMAIL", "agentconductor@localhost");
        for name in [
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_INDEX_FILE",
            "GIT_COMMON_DIR",
            "GIT_OBJECT_DIRECTORY",
            "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        ] {
            command.env_remove(name);
        }
        if let Some(index) = index {
            command.env("GIT_INDEX_FILE", git_path(index.as_os_str()));
        }
        let mut child = command.spawn().map_err(|e| e.to_string())?;
        let mut stdin = child.stdin.take().ok_or("Git stdin missing")?;
        let bytes = input.unwrap_or_default().to_vec();
        let writer = std::thread::spawn(move || stdin.write_all(&bytes));
        let output = child.wait_with_output().map_err(|e| e.to_string())?;
        writer
            .join()
            .map_err(|_| "Git input writer panicked")?
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(format!(
                "git {}: {}",
                args.first().unwrap_or(&""),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(output.stdout)
    }
    pub fn command(&self, args: &[&str]) -> Result<Vec<u8>, String> {
        self.invoke(args, None, None)
    }
    pub fn text(&self, args: &[&str]) -> Result<String, String> {
        String::from_utf8(self.command(args)?)
            .map(|s| s.trim().into())
            .map_err(|e| e.to_string())
    }
    pub fn head(&self) -> Result<String, String> {
        self.text(&["rev-parse", "HEAD"])
    }
    pub fn branch(&self) -> Result<String, String> {
        self.text(&["symbolic-ref", "HEAD"])
    }
    /// Writes the regular files tracked at `commit` into a new directory `dest`.
    /// Untracked/ignored files, symlinks and submodules are never materialized.
    pub fn export_tree(&self, commit: &str, dest: &Path) -> Result<(), String> {
        let listing = self.command(&["ls-tree", "-r", "-z", "--full-tree", commit])?;
        let mut files = Vec::new();
        for entry in listing.split(|b| *b == 0).filter(|e| !e.is_empty()) {
            let entry = std::str::from_utf8(entry).map_err(|e| e.to_string())?;
            let (meta, path) = entry.split_once('\t').ok_or("malformed tree entry")?;
            let mut fields = meta.split(' ');
            let (mode, kind, id) = (fields.next(), fields.next(), fields.next());
            if kind != Some("blob") || !matches!(mode, Some("100644" | "100755")) {
                continue;
            }
            if path
                .split('/')
                .any(|c| c.is_empty() || c == "." || c == "..")
            {
                return Err("unsafe tracked path".into());
            }
            files.push((
                path.to_owned(),
                id.ok_or("malformed tree entry")?.to_owned(),
                mode == Some("100755"),
            ));
        }
        let request: String = files.iter().map(|(_, id, _)| format!("{id}\n")).collect();
        let objects = self.invoke(&["cat-file", "--batch"], Some(request.as_bytes()), None)?;
        std::fs::create_dir(dest).map_err(|e| e.to_string())?;
        let mut rest = objects.as_slice();
        for (path, id, executable) in files {
            let end = rest
                .iter()
                .position(|b| *b == b'\n')
                .ok_or("truncated object stream")?;
            let header = std::str::from_utf8(&rest[..end]).map_err(|e| e.to_string())?;
            let size: usize = match header.split(' ').collect::<Vec<_>>()[..] {
                [object, "blob", size] if object == id => size.parse().map_err(|_| "bad size")?,
                _ => return Err("unexpected object in stream".into()),
            };
            let body = rest
                .get(end + 1..end + 1 + size)
                .ok_or("truncated object stream")?;
            rest = &rest[(end + 2 + size).min(rest.len())..];
            let target = dest.join(&path);
            std::fs::create_dir_all(target.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::write(&target, body).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            if executable {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o755))
                    .map_err(|e| e.to_string())?;
            }
            #[cfg(not(unix))]
            let _ = executable;
        }
        Ok(())
    }
    pub fn clean(&self) -> Result<(), String> {
        if !self
            .command(&["status", "--porcelain", "--untracked-files=all"])?
            .is_empty()
        {
            return Err("a clean repository baseline is required".into());
        }
        Ok(())
    }
    pub fn worktree(&self, path: &Path, base: &str) -> Result<(), String> {
        self.command(&[
            "worktree",
            "add",
            "--detach",
            path.to_str().ok_or("non-UTF8 worktree path")?,
            base,
        ])?;
        Ok(())
    }
    pub fn changed(&self, base: &str) -> Result<Vec<String>, String> {
        let mut paths = BTreeSet::new();
        for bytes in [
            self.command(&["diff", "--no-renames", "--name-only", "-z", base, "--"])?,
            self.command(&["ls-files", "--others", "--exclude-standard", "-z"])?,
        ] {
            for part in bytes.split(|b| *b == 0).filter(|p| !p.is_empty()) {
                paths.insert(String::from_utf8(part.to_vec()).map_err(|e| e.to_string())?);
            }
        }
        Ok(paths.into_iter().collect())
    }
    pub fn check_scope(&self, changed: &[String], scopes: &[String]) -> Result<(), String> {
        for path in changed {
            if !scopes.iter().any(|scope| {
                overlaps(path, scope) && (path == scope || path.starts_with(&format!("{scope}/")))
            }) {
                return Err(format!("out-of-scope change: {path}"));
            }
            let mut current = self.root.clone();
            for component in Path::new(path).components() {
                current.push(component);
                if std::fs::symlink_metadata(&current).is_ok_and(|m| m.file_type().is_symlink()) {
                    let target = current.canonicalize().map_err(|e| e.to_string())?;
                    if !target.starts_with(&self.root) {
                        return Err(format!("symlink escapes workspace: {path}"));
                    }
                }
            }
        }
        Ok(())
    }
}

/// Git for Windows cannot use extended-length (`\\?\`) paths that `canonicalize`
/// returns; convert them to ordinary drive or UNC paths. Other values pass through.
pub fn git_path(value: &std::ffi::OsStr) -> std::ffi::OsString {
    #[cfg(windows)]
    if let Some(text) = value.to_str() {
        if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{rest}").into();
        }
        if let Some(rest) = text.strip_prefix(r"\\?\")
            && rest.as_bytes().get(1) == Some(&b':')
        {
            return rest.into();
        }
    }
    value.to_os_string()
}
