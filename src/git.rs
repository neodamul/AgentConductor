use crate::plan::overlaps;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Clone)]
pub struct Repository {
    pub root: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub commit: String,
    pub tree: String,
    pub changed: Vec<String>,
}

impl Repository {
    pub fn open(path: &Path) -> Result<Self, String> {
        let repo = Self {
            root: path.canonicalize().map_err(|e| e.to_string())?,
        };
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
    fn invoke(
        &self,
        args: &[&str],
        input: Option<&[u8]>,
        index: Option<&Path>,
    ) -> Result<Vec<u8>, String> {
        let mut command = Command::new("git");
        command
            .arg("-C")
            .arg(&self.root)
            .args([
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.fsmonitor=false",
            ])
            .args(args)
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
            command.env("GIT_INDEX_FILE", index);
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
    pub fn snapshot(
        &self,
        base: &str,
        scopes: &[String],
        index: &Path,
    ) -> Result<Snapshot, String> {
        let changed = self.changed(base)?;
        self.check_scope(&changed, scopes)?;
        if index.exists() {
            return Err("temporary index already exists".into());
        }
        let result = (|| {
            self.invoke(&["read-tree", base], None, Some(index))?;
            let mut args = vec!["add", "-A", "--"];
            args.extend(changed.iter().map(String::as_str));
            if !changed.is_empty() {
                self.invoke(&args, None, Some(index))?;
            }
            let tree = String::from_utf8(self.invoke(&["write-tree"], None, Some(index))?)
                .map_err(|e| e.to_string())?
                .trim()
                .to_string();
            let commit = String::from_utf8(self.invoke(
                &["commit-tree", &tree, "-p", base],
                Some(b"AgentConductor candidate\n"),
                Some(index),
            )?)
            .map_err(|e| e.to_string())?
            .trim()
            .to_string();
            Ok(Snapshot {
                commit,
                tree,
                changed,
            })
        })();
        let _ = std::fs::remove_file(index);
        result
    }
    pub fn diff(&self, base: &str, candidate: &str) -> Result<Vec<u8>, String> {
        self.command(&["diff", "--binary", "--no-renames", base, candidate, "--"])
    }
    pub fn combine(
        &self,
        base: &str,
        patches: &[Vec<u8>],
        index: &Path,
    ) -> Result<Snapshot, String> {
        if index.exists() {
            return Err("temporary index already exists".into());
        }
        let result = (|| {
            self.invoke(&["read-tree", base], None, Some(index))?;
            for patch in patches {
                if !patch.is_empty() {
                    self.invoke(
                        &["apply", "--cached", "--binary", "-"],
                        Some(patch),
                        Some(index),
                    )?;
                }
            }
            let tree = String::from_utf8(self.invoke(&["write-tree"], None, Some(index))?)
                .map_err(|e| e.to_string())?
                .trim()
                .to_string();
            let commit = String::from_utf8(self.invoke(
                &["commit-tree", &tree, "-p", base],
                Some(b"AgentConductor integration candidate\n"),
                Some(index),
            )?)
            .map_err(|e| e.to_string())?
            .trim()
            .to_string();
            let names = self.text(&["diff", "--no-renames", "--name-only", base, &commit, "--"])?;
            Ok(Snapshot {
                commit,
                tree,
                changed: names.lines().map(String::from).collect(),
            })
        })();
        let _ = std::fs::remove_file(index);
        result
    }
    pub fn merge_shared(&self, branch: &str, base: &str, candidate: &str) -> Result<(), String> {
        self.command(&["update-ref", branch, candidate, base])?;
        self.command(&["read-tree", candidate])?;
        Ok(())
    }
}
