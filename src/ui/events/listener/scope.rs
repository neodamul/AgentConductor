use super::FileStamp;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::{Component, Path, PathBuf},
};

const MAX_SCOPE_PATHS: usize = 512;
const MAX_SCOPE_DEPTH: usize = 8;

#[cfg(test)]
#[path = "../../../../tests/support/worktree_scope.rs"]
mod tests;

pub(super) fn declared_paths(report_path: &Path) -> Vec<PathBuf> {
    let Ok(report) = fs::read(report_path) else {
        return Vec::new();
    };
    let Ok(report) = serde_json::from_slice::<Value>(&report) else {
        return Vec::new();
    };
    let worktrees = report
        .get("tasks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|task| {
            Some((
                task.get("id")?.as_str()?.to_owned(),
                PathBuf::from(task.get("cwd")?.as_str()?),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let mut paths = BTreeSet::new();
    for task in report
        .pointer("/plan/tasks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(root) = task
            .get("id")
            .and_then(Value::as_str)
            .and_then(|id| worktrees.get(id))
        else {
            continue;
        };
        for relative in task
            .get("paths")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter_map(safe_scope_path)
        {
            collect_tree(&root.join(relative), &mut paths);
            if paths.len() >= MAX_SCOPE_PATHS {
                return paths.into_iter().collect();
            }
        }
    }
    paths.into_iter().collect()
}

fn safe_scope_path(path: &str) -> Option<PathBuf> {
    let path = path.replace('\\', "/");
    let wildcard = path.find(['*', '?', '[']).unwrap_or(path.len());
    let prefix = &path[..wildcard];
    let prefix = prefix.rsplit_once('/').map_or(prefix, |(parent, _)| parent);
    let relative = Path::new(if prefix.is_empty() { "." } else { prefix });
    let drive_prefix =
        prefix.as_bytes().get(1) == Some(&b':') && prefix.as_bytes()[0].is_ascii_alphabetic();
    if drive_prefix
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return None;
    }
    Some(relative.to_path_buf())
}

fn collect_tree(root: &Path, paths: &mut BTreeSet<PathBuf>) {
    let mut pending = VecDeque::from([(root.to_path_buf(), 0)]);
    while let Some((path, depth)) = pending.pop_front() {
        if paths.len() >= MAX_SCOPE_PATHS || !paths.insert(path.clone()) || depth > MAX_SCOPE_DEPTH
        {
            continue;
        }
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            continue;
        }
        let Ok(entries) = fs::read_dir(path) else {
            continue;
        };
        for entry in entries.flatten() {
            if paths.len() + pending.len() >= MAX_SCOPE_PATHS {
                return;
            }
            pending.push_back((entry.path(), depth + 1));
        }
    }
}

pub(super) fn stamp(path: &Path) -> Option<FileStamp> {
    fs::symlink_metadata(path).ok().map(|metadata| FileStamp {
        length: metadata.len(),
        modified: metadata.modified().ok(),
    })
}
