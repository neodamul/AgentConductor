use super::*;
impl Source {
    /// Preserve Git objects and run evidence; discard only completed working files.
    pub(crate) fn compact(&self, run: &Path, working: &[PathBuf]) -> Result<(), String> {
        let run = run.canonicalize().map_err(|e| e.to_string())?;
        let root = self.workspace.canonicalize().map_err(|e| e.to_string())?;
        if root != self.workspace || root == self.original {
            return Err("private cleanup binding changed".into());
        }
        let mut paths = std::collections::BTreeSet::from([root]);
        for path in working {
            let path = path.canonicalize().map_err(|e| e.to_string())?;
            if !path.starts_with(&run) && path != self.workspace {
                return Err("working-copy cleanup outside owned run".into());
            }
            paths.insert(path);
        }
        let mut leases = Vec::new();
        for path in &paths {
            if path != &self.workspace {
                leases.push(crate::leases::WorkspaceLease::acquire(path, &run)?);
            }
        }
        for path in paths {
            if path == self.original {
                return Err("cannot compact original project".into());
            }
            let _repo = Repository::open(&path)?;
            for entry in fs::read_dir(&path).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                if entry.file_name() == ".git" {
                    continue;
                }
                let metadata = fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
                if metadata.is_dir() && ordinary(&metadata) {
                    fs::remove_dir_all(entry.path()).map_err(|e| e.to_string())?;
                } else if metadata.is_dir() {
                    fs::remove_dir(entry.path()).map_err(|e| e.to_string())?;
                } else {
                    fs::remove_file(entry.path()).map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }
}
pub(super) struct Stage(pub PathBuf);
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
