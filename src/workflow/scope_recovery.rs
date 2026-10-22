//! Reviewed recovery from an out-of-scope integration failure.
use super::*;
use serde_json::{Value, json};
use std::path::Path;

mod paths;
use paths::{quarantined, restore_exclusive, safe_target, within};

/// Describes the failed scope check, task permissions, and recoverable files.
pub fn scope_failure(dir: &Path) -> Result<Value, String> {
    let report = load(dir)?;
    if !report
        .error
        .as_deref()
        .is_some_and(|error| error.starts_with("out-of-scope change:"))
    {
        return Err("the run has no out-of-scope failure to recover".into());
    }
    let mut offending = Vec::new();
    let mut inspect = |git: &Repository, base: &str, owned: &[String]| {
        for path in git.changed(base)? {
            if !owned.iter().any(|scope| within(&path, scope)) && !offending.contains(&path) {
                offending.push(path);
            }
        }
        Ok::<_, String>(())
    };
    if report.plan.workspace_mode == WorkspaceMode::Worktree {
        for (task, spec) in report.tasks.iter().zip(&report.plan.tasks) {
            if task.cwd.is_dir() {
                inspect(&Repository::open(&task.cwd)?, &task.base, &spec.paths)?;
            }
        }
    } else {
        inspect(
            &Repository::open(&report.repo)?,
            &report.base,
            &scopes(&report.plan),
        )?;
    }
    let store = Store::open(dir.join("state.sqlite"))?;
    let mut stray = Vec::new();
    for event in store.audit("stray_files_moved", i64::MAX as usize)? {
        for path in event["paths"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            let stored = quarantined(dir, path)?;
            if fs::symlink_metadata(&stored).is_ok_and(|metadata| metadata.is_file())
                && !stray.iter().any(|item: &Value| item["path"] == path)
            {
                stray.push(json!({
                    "path": path,
                    "stored": stored,
                    "exists_at_original": safe_target(&report.repo, path)
                        .is_ok_and(|original| original.exists())
                }));
            }
        }
    }
    Ok(json!({
        "error": report.error,
        "offending": offending,
        "tasks": report.plan.tasks.iter().map(|task| json!({
            "id": task.id, "paths": task.paths, "test_command": task.test_command
        })).collect::<Vec<_>>(),
        "stray": stray,
        "note": "Feedback never changes owned paths. Approve a reviewed plan change to continue; quarantined files stay available until you choose to restore them.",
    }))
}

struct Proposal {
    plan: Plan,
    value: Value,
}

fn proposal(
    dir: &Path,
    task: &str,
    paths: &[String],
    test: Option<&[String]>,
) -> Result<Proposal, String> {
    let report = load(dir)?;
    editable(&report)?;
    let index = task_index(&report, task)?;
    let paths: Vec<String> = paths
        .iter()
        .map(|path| path.trim().trim_end_matches('/').to_owned())
        .collect();
    if paths.is_empty() {
        return Err("a task needs at least one owned path".into());
    }
    let mut plan = report.plan.clone();
    plan.tasks[index].paths = paths.clone();
    if let Some(test) = test {
        plan.tasks[index].test_command = test.to_vec();
    }
    plan.validate()?;
    let root = Repository::open(&report.repo)?.root;
    let store = Store::open(dir.join("state.sqlite"))?;
    let mut restore = Vec::new();
    for event in store.audit("stray_files_moved", i64::MAX as usize)? {
        for path in event["paths"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if restore.iter().any(|item: &Value| item["path"] == path)
                || !scopes(&plan).iter().any(|scope| within(path, scope))
            {
                continue;
            }
            let stored = quarantined(dir, path)?;
            if !fs::symlink_metadata(&stored).is_ok_and(|metadata| metadata.is_file()) {
                continue;
            }
            let destination = safe_target(&root, path)?;
            let action = match fs::symlink_metadata(&destination) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => "restore",
                Err(error) => return Err(error.to_string()),
                Ok(_)
                    if fs::read(&destination).map_err(|e| e.to_string())?
                        == fs::read(&stored).map_err(|e| e.to_string())? =>
                {
                    "already_present"
                }
                Ok(_) => "conflict",
            };
            restore.push(json!({"path":path,"action":action}));
        }
    }
    let value = json!({
        "task": task,
        "paths": paths,
        "previous_paths": report.plan.tasks[index].paths,
        "test_command": plan.tasks[index].test_command,
        "previous_test_command": report.plan.tasks[index].test_command,
        "restore": restore,
    });
    Ok(Proposal { plan, value })
}

/// Returns the exact plan and restoration changes that require operator approval.
pub fn recovery_preview(
    dir: &Path,
    task: &str,
    paths: &[String],
    test: Option<Vec<String>>,
) -> Result<Value, String> {
    let mut value = proposal(dir, task, paths, test.as_deref())?.value;
    value["digest"] = digest(&serde_json::to_vec(&value).map_err(|e| e.to_string())?).into();
    Ok(value)
}

/// Applies only the reviewed digest; the existing Resume action continues the stopped run.
pub fn recovery_apply(
    dir: &Path,
    task: &str,
    paths: &[String],
    test: Option<Vec<String>>,
    approved: &str,
) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let current = recovery_preview(dir, task, paths, test.clone())?;
    if current["digest"] != approved {
        return Err("the proposal changed or was not reviewed; preview it again".into());
    }
    let Proposal { plan, .. } = proposal(dir, task, paths, test.as_deref())?;
    let mut report = load(dir)?;
    let root = Repository::open(&report.repo)?.root;
    let mut restored = Vec::new();
    for item in current["restore"].as_array().into_iter().flatten() {
        if item["action"] != "restore" {
            continue;
        }
        let path = item["path"]
            .as_str()
            .ok_or("invalid restore proposal path")?;
        let from = quarantined(dir, path)?;
        let to = safe_target(&root, path)?;
        let parent = to.parent().ok_or("restore destination has no parent")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        safe_target(&root, path)?;
        restore_exclusive(&from, &to)?;
        restored.push(path.to_owned());
    }
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record(
        "scope_recovery_approved",
        &json!({
            "proposal":current,
            "restored":restored,
            "previous_plan_digest":digest(&serde_json::to_vec(&report.plan).map_err(|e|e.to_string())?)
        }),
        "Operator approved a reviewed scope change; finished work is reused and integration verification reruns",
    )?;
    report.plan = plan;
    report.candidate = None;
    report.diff_hash = None;
    report.tests.clear();
    report.error = None;
    report.state = "stopped".into();
    store.record(
        "approval",
        &Value::Null,
        "Scope recovery revoked prior integration approval",
    )?;
    save(&mut store, dir, &report)
}
