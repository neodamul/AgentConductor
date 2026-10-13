//! DD-133: record completed workers' owned files and warn about later edits.
use super::*;
use crate::{
    state::Store,
    workflow::{Report, TaskReport},
};
fn modified(path: &Path) -> Option<u128> {
    fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_nanos())
}
fn source(root: &Path) -> Source {
    Source {
        original: root.to_path_buf(),
        workspace: root.to_path_buf(),
        base: String::new(),
        files: BTreeMap::new(),
    }
}

fn owned_names(
    bound: &Source,
    scopes: &[String],
) -> Result<std::collections::BTreeSet<String>, String> {
    fn visit(
        bound: &Source,
        relative: &str,
        out: &mut std::collections::BTreeSet<String>,
    ) -> Result<(), String> {
        let path = bound.safe_path(relative)?;
        let metadata = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.to_string()),
        };
        if metadata.is_dir() {
            for entry in fs::read_dir(&path).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| "project paths must be UTF-8")?;
                if excluded(&name)
                    || entry.path().is_dir() && entry.path().join(crate::slugs::MARKER).is_file()
                {
                    continue;
                }
                visit(bound, &format!("{relative}/{name}"), out)?;
            }
        } else if metadata.is_file() {
            out.insert(relative.to_string());
        }
        if out.len() > MAX_FILES {
            return Err("owned file set exceeds 10,000 files".into());
        }
        Ok(())
    }
    let mut names = std::collections::BTreeSet::new();
    for scope in scopes {
        visit(bound, scope, &mut names)?;
    }
    Ok(names)
}
pub fn record_owned_checkpoint(
    store: &mut Store,
    git: &Repository,
    task: &TaskReport,
    scopes: &[String],
) -> Result<(), String> {
    if !git.is_external() {
        return Ok(());
    }
    let bound = source(&git.root);
    let mut paths = std::collections::BTreeSet::new();
    let mut args = vec![
        "ls-tree",
        "-r",
        "-z",
        "--name-only",
        task.base.as_str(),
        "--",
    ];
    args.extend(scopes.iter().map(String::as_str));
    for name in git
        .command(&args)?
        .split(|b| *b == 0)
        .filter(|b| !b.is_empty())
    {
        paths.insert(
            std::str::from_utf8(name)
                .map_err(|e| e.to_string())?
                .to_string(),
        );
    }
    paths.extend(owned_names(&bound, scopes)?);
    let files = paths
        .iter()
        .map(|path| {
            let file = bound.safe_path(path)?;
            Ok((
                path.clone(),
                serde_json::json!({"state":state(&file)?,"modified_ns":modified(&file)}),
            ))
        })
        .collect::<Result<serde_json::Map<String, serde_json::Value>, String>>()?;
    store.record(
        &format!("owned_checkpoint/{}", task.id),
        &serde_json::json!({"files":files,"baseline":task.base,"scopes":scopes}),
        "Recorded owned files at worker completion",
    )
}
pub fn owned_changes(store: &Store, report: &Report) -> Result<Vec<String>, String> {
    if matches!(report.state.as_str(), "completed" | "rejected")
        || report.repo.join(".git").exists()
        || external_metadata(&report.repo)?.is_none()
    {
        return Ok(vec![]);
    }
    let bound = source(&report.repo);
    let mut changed = std::collections::BTreeSet::new();
    for task in report.tasks.iter().filter(|t| t.status == "completed") {
        let Some(checkpoint) = store.optional_document(&format!("owned_checkpoint/{}", task.id))?
        else {
            continue;
        };
        let Some(files) = checkpoint["files"].as_object() else {
            continue;
        };
        if let Some(scopes) = checkpoint["scopes"].as_array() {
            let scopes = scopes
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>();
            for path in owned_names(&bound, &scopes)? {
                if !files.contains_key(&path) {
                    changed.insert(path);
                }
            }
        }
        for (path, expected) in files {
            let file = bound.safe_path(path)?;
            if serde_json::json!(modified(&file)) != expected["modified_ns"]
                && serde_json::to_value(state(&file)?).map_err(|e| e.to_string())?
                    != expected["state"]
            {
                changed.insert(path.clone());
            }
        }
    }
    Ok(changed.into_iter().collect())
}
