use super::*;
use std::io::Write;

impl Source {
    /// Called only by the workflow's exact approved-candidate application path.
    pub fn apply(
        &self,
        git: &Repository,
        candidate: &crate::git::Snapshot,
        run: &Path,
    ) -> Result<(), String> {
        let run = run.canonicalize().map_err(|e| e.to_string())?;
        self.validate()?;
        let journal = run.join("folder-apply");
        if journal.exists() {
            return Err(format!(
                "folder application receipt already exists; inspect {} before retrying",
                journal.display()
            ));
        }
        let stage = run.join(format!(
            "folder-candidate-{}",
            crate::native_guard::session_id()?
        ));
        git.export_tree(&candidate.commit, &stage)?;
        let mut changes = Vec::new();
        for path in &candidate.changed {
            let target = self.safe_path(path)?;
            let expected = self.files.get(path).cloned();
            if state(&target)? != expected {
                return Err("original folder changed before application".into());
            }
            let next = state(&stage.join(path))?;
            changes.push((path.clone(), expected, next));
        }
        fs::create_dir(&journal).map_err(|e| e.to_string())?;
        let backup = journal.join("backup");
        for (path, old, _) in &changes {
            if old.is_some() {
                let dest = backup.join(path);
                fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
                fs::copy(self.original.join(path), &dest).map_err(|e| e.to_string())?;
                fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(dest)
                    .and_then(|f| f.sync_all())
                    .map_err(|e| e.to_string())?;
            }
        }
        let receipt = serde_json::json!({"original":self.original,"candidate":candidate,"changes":changes,"state":"applying"});
        fs::write(
            journal.join("receipt.json"),
            serde_json::to_vec_pretty(&receipt).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(journal.join("receipt.json"))
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())?;
        crate::process::sync_directory(&journal)?;
        self.validate()?;
        let mut applied: Vec<usize> = Vec::new();
        let result = (|| {
            for (i, (path, old, next)) in changes.iter().enumerate() {
                let target = self.safe_path(path)?;
                if state(&target)? != *old {
                    return Err("original folder changed during application".to_string());
                }
                match next {
                    Some(_) => replace(&stage.join(path), &target)?,
                    None => fs::remove_file(&target).map_err(|e| e.to_string())?,
                }
                applied.push(i);
            }
            Ok::<_, String>(())
        })();
        if let Err(error) = result {
            for &i in applied.iter().rev() {
                let (path, old, next) = &changes[i];
                let target = self.safe_path(path)?;
                if state(&target)? != *next {
                    return Err(format!(
                        "{error}; concurrent edit prevents rollback; backups: {}",
                        backup.display()
                    ));
                }
                if old.is_some() {
                    replace(&backup.join(path), &target)?;
                } else {
                    fs::remove_file(target).map_err(|e| e.to_string())?;
                }
            }
            fs::write(journal.join("ROLLED_BACK"), &error).map_err(|e| e.to_string())?;
            return Err(format!(
                "{error}; original files restored; inspect {}",
                journal.display()
            ));
        }
        fs::write(journal.join("COMPLETED"), candidate.commit.as_bytes())
            .map_err(|e| e.to_string())?;
        crate::process::sync_directory(&journal)?;
        fs::remove_dir_all(stage).map_err(|e| e.to_string())?;
        Ok(())
    }
}
fn replace(source: &Path, target: &Path) -> Result<(), String> {
    let parent = target.parent().ok_or("missing folder parent")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temporary = parent.join(format!(".ac-apply-{}", crate::native_guard::session_id()?));
    let result = (|| {
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| e.to_string())?;
        let mut input = crate::ui::open_artifact(source)?;
        std::io::copy(&mut input, &mut output).map_err(|e| e.to_string())?;
        output
            .flush()
            .and_then(|_| output.sync_all())
            .map_err(|e| e.to_string())?;
        fs::set_permissions(
            &temporary,
            fs::metadata(source)
                .map_err(|e| e.to_string())?
                .permissions(),
        )
        .map_err(|e| e.to_string())?;
        drop(output);
        fs::rename(&temporary, target).map_err(|e| e.to_string())?;
        crate::process::sync_directory(parent)
    })();
    if temporary.exists() {
        let _ = fs::remove_file(temporary);
    }
    result
}
