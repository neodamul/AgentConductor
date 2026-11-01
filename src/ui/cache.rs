//! Disposable metadata caches only; reviewed work and recovery data are never candidates.
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

#[derive(Serialize)]
struct Entry {
    path: PathBuf,
    bytes: u64,
    modified: u128,
    fingerprint: String,
}
fn terminal(path: &Path) -> bool {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return false;
    };
    if !crate::folder::ordinary(&meta) || !meta.is_file() || meta.len() > 16 * 1024 {
        return false;
    }
    let Ok(bytes) = fs::read(path) else {
        return false;
    };
    let Ok(owner) = serde_json::from_slice::<crate::process::Ownership>(&bytes) else {
        return false;
    };
    if owner.phase != "terminal"
        || owner.pid.is_none_or(|p| p == 0 || p > i32::MAX as u32)
        || owner.process_group != owner.pid.map(|p| p as i32)
    {
        return false;
    }
    owner.exit_code == Some(0) || owner.ensure_quiescent().is_ok()
}
fn tree(
    path: &Path,
    rows: &mut Vec<(PathBuf, u64, u128)>,
    budget: &mut usize,
) -> Result<u64, String> {
    if rows.len() >= 10_000 || *budget == 0 {
        return Err(crate::ui::messages::text("native.cache_scan_exceeds_bound").into());
    }
    *budget -= 1;
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !crate::folder::ordinary(&metadata) {
        return Err(crate::ui::messages::text("native.aliased_cache_retained").into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.is_file() && metadata.nlink() != 1 {
            return Err(crate::ui::messages::text("native.hard_linked_cache_retained").into());
        }
    }
    let modified = metadata
        .modified()
        .map_err(|e| e.to_string())?
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    rows.push((path.to_owned(), metadata.len(), modified));
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    if !metadata.is_dir() {
        return Err(crate::ui::messages::text("native.special_cache_retained").into());
    }
    let mut bytes = 0;
    for item in fs::read_dir(path).map_err(|e| e.to_string())? {
        bytes += tree(&item.map_err(|e| e.to_string())?.path(), rows, budget)?;
    }
    Ok(bytes)
}
fn inventory(root: &Path, protected: &[PathBuf]) -> Result<(Vec<Entry>, usize), String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut paths = Vec::new();
    let mut skipped = 0;
    for (parent, prefix) in [
        (root.clone(), "quota-query-"),
        (root.join("quota-jobs"), "coordinator-ui-"),
        (root.join("quota-jobs"), "catalog-"),
        (root.join("chat-jobs"), "coordinator-ui-"),
    ] {
        if !parent.exists() {
            continue;
        }
        if !crate::folder::ordinary(&fs::symlink_metadata(&parent).map_err(|e| e.to_string())?) {
            continue;
        }
        for item in fs::read_dir(&parent).map_err(|e| e.to_string())? {
            let item = item.map_err(|e| e.to_string())?;
            if item.file_name().to_string_lossy().starts_with(prefix) {
                paths.push((item.path(), prefix));
            }
        }
    }
    if paths.len() > 20_000 {
        return Err(crate::ui::messages::text("native.cache_inventory_exceeds_bound").into());
    }
    let pids: Vec<_> = protected
        .iter()
        .filter_map(|p| fs::read(p.join("process/owner.json")).ok())
        .filter_map(|b| serde_json::from_slice::<crate::process::Ownership>(&b).ok())
        .filter_map(|o| o.pid)
        .collect();
    let mut entries = Vec::new();
    let mut budget = 20_000usize;
    for (path, kind) in paths {
        if budget == 0 {
            skipped += 1;
            continue;
        }
        if protected
            .iter()
            .any(|p| p.starts_with(&path) || path.starts_with(p))
        {
            skipped += 1;
            continue;
        }
        let owner = if kind == "coordinator-ui-" {
            path.join("process/owner.json")
        } else {
            path.join("owner.json")
        };
        let eligible = if kind == "catalog-" {
            fs::read(path.join("cache-owner.json"))
                .ok()
                .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
                .and_then(|v| v["job"].as_str().map(str::to_owned))
                .filter(|id| {
                    !id.is_empty() && id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
                })
                .is_some_and(|id| {
                    terminal(
                        &root.join(format!("quota-jobs/coordinator-ui-{id}/process/owner.json")),
                    )
                })
        } else {
            terminal(&owner)
                && !fs::read(&owner)
                    .ok()
                    .and_then(|b| serde_json::from_slice::<crate::process::Ownership>(&b).ok())
                    .is_some_and(|o| kind == "quota-query-" && pids.contains(&o.coordinator_pid))
        };
        let parent_done = kind != "quota-query-"
            || fs::read(&owner)
                .ok()
                .and_then(|b| serde_json::from_slice::<crate::process::Ownership>(&b).ok())
                .is_some_and(|o| {
                    let parent = crate::process::Ownership {
                        coordinator_pid: o.coordinator_pid,
                        pid: Some(o.coordinator_pid),
                        process_group: i32::try_from(o.coordinator_pid).ok(),
                        phase: "terminal".into(),
                        exit_code: Some(0),
                        termination_signal: None,
                        stopped: false,
                        timed_out: false,
                    };
                    parent.ensure_quiescent().is_ok()
                });
        if !eligible || !parent_done {
            skipped += 1;
            continue;
        }
        let mut rows = Vec::new();
        let Ok(bytes) = tree(&path, &mut rows, &mut budget) else {
            skipped += 1;
            continue;
        };
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        let modified = rows.iter().map(|r| r.2).max().unwrap_or(0);
        let fingerprint =
            crate::workflow::digest(&serde_json::to_vec(&rows).map_err(|e| e.to_string())?);
        entries.push(Entry {
            path,
            bytes,
            modified,
            fingerprint,
        });
    }
    entries.sort_by(|a, b| b.modified.cmp(&a.modified).then(a.path.cmp(&b.path)));
    Ok((entries, skipped))
}
pub(super) fn preview(root: &Path, protected: &[PathBuf]) -> Result<serde_json::Value, String> {
    let (entries, skipped) = inventory(root, protected)?;
    let digest = crate::workflow::digest(&serde_json::to_vec(&entries).map_err(|e| e.to_string())?);
    Ok(
        serde_json::json!({"digest":digest,"bytes":entries.iter().map(|e|e.bytes).sum::<u64>(),"items":entries,"skipped":skipped}),
    )
}
pub(super) fn clear(
    root: &Path,
    protected: &[PathBuf],
    expected: Option<&str>,
    keep: usize,
) -> Result<serde_json::Value, String> {
    let preview = preview(root, protected)?;
    if expected.is_some_and(|d| preview["digest"] != d) {
        return Err(crate::ui::messages::text(
            "native.cache_changed_since_preview_review_it_again",
        )
        .into());
    }
    let mut removed = 0;
    let mut bytes = 0;
    for item in preview["items"]
        .as_array()
        .ok_or(crate::ui::messages::text("native.invalid_cache_inventory"))?
        .iter()
        .skip(keep)
    {
        let path = Path::new(
            item["path"]
                .as_str()
                .ok_or(crate::ui::messages::text("native.invalid_cache_path"))?,
        );
        let mut rows = Vec::new();
        tree(path, &mut rows, &mut 10_000usize)?;
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        if crate::workflow::digest(&serde_json::to_vec(&rows).map_err(|e| e.to_string())?)
            != item["fingerprint"]
        {
            return Err(crate::ui::messages::text(
                "native.cache_changed_while_clearing_remaining_cache_retained",
            )
            .into());
        }
        crate::workflow::release_run_worktrees(path);
        fs::remove_dir_all(path).map_err(|e| e.to_string())?;
        removed += 1;
        bytes += item["bytes"].as_u64().unwrap_or(0);
    }
    Ok(serde_json::json!({"removed":removed,"bytes":bytes}))
}
