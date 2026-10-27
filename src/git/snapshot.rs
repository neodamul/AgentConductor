use super::*;

impl Repository {
    pub fn snapshot(
        &self,
        base: &str,
        scopes: &[String],
        index: &Path,
    ) -> Result<Snapshot, String> {
        let changed = self.changed(base)?;
        self.check_scope(&changed, scopes)?;
        self.snapshot_paths(base, changed, index)
    }
    /// Records the whole current workspace state without a scope check. Used for
    /// approval fingerprints while other owned-scope writers may still be active.
    pub fn checkpoint(&self, base: &str, index: &Path) -> Result<Snapshot, String> {
        let changed = self.changed(base)?;
        self.snapshot_paths(base, changed, index)
    }
    fn snapshot_paths(
        &self,
        base: &str,
        changed: Vec<String>,
        index: &Path,
    ) -> Result<Snapshot, String> {
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
    pub fn apply_working_patch(&self, patch: &[u8]) -> Result<(), String> {
        if !patch.is_empty() {
            self.invoke(&["apply", "--check", "--binary", "-"], Some(patch), None)?;
            self.invoke(&["apply", "--binary", "-"], Some(patch), None)?;
        }
        Ok(())
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
