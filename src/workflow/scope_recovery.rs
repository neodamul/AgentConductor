//! Guided recovery from an out-of-scope failure (GitHub #122, DD-172). Feedback never changes a
//! task's owned paths; the operator reviews an exact plan change (owned paths, test command) and
//! the files to restore from `stray-files/`, approves its digest, and the run resumes from its
//! finished work with the integration approval revoked.
use super::*;
use serde_json::{Value, json};

fn within(path: &str, scope: &str) -> bool {
    let scope = scope.trim_end_matches('/');
    path == scope || path.starts_with(&format!("{scope}/"))
}

/// Why the run failed: every offending file, each task's approved paths and the quarantine.
pub fn scope_failure(dir: &Path) -> Result<Value, String> {
    let report = load(dir)?;
    if !report
        .error
        .as_deref()
        .is_some_and(|e| e.starts_with("out-of-scope change:"))
    {
        return Err("the run has no out-of-scope failure to recover".into());
    }
    let mut offending = Vec::new();
    let worktree = report.plan.workspace_mode == WorkspaceMode::Worktree;
    let mut check = |git: &Repository, base: &str, scopes: &[String]| -> Result<(), String> {
        for path in git.changed(base)? {
            if !scopes.iter().any(|s| within(&path, s)) && !offending.contains(&path) {
                offending.push(path);
            }
        }
        Ok(())
    };
    if worktree {
        for (task, spec) in report.tasks.iter().zip(&report.plan.tasks) {
            if task.cwd.is_dir() {
                check(&Repository::open(&task.cwd)?, &task.base, &spec.paths)?;
            }
        }
    } else {
        check(
            &Repository::open(&report.repo)?,
            &report.base,
            &scopes(&report.plan),
        )?;
    }
    let store = Store::open(dir.join("state.sqlite"))?;
    let mut stray = Vec::new();
    for event in store.audit("stray_files_moved", 50)? {
        for path in event["paths"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            let stored = dir.join("stray-files").join(path);
            if stored.is_file() && !stray.iter().any(|s: &Value| s["path"] == path) {
                let original = report.repo.join(path);
                stray.push(json!({"path": path, "stored": stored, "exists_at_original": original.exists()}));
            }
        }
    }
    Ok(json!({
        "error": report.error,
        "offending": offending,
        "tasks": report.plan.tasks.iter().map(|t| json!({"id": t.id, "paths": t.paths, "test_command": t.test_command})).collect::<Vec<_>>(),
        "stray": stray,
        "note": "Sending feedback never changes a task's owned paths. Approve a reviewed plan change to continue; moved files are kept in stray-files and restored only when you choose.",
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
        .map(|p| p.trim().trim_end_matches('/').to_owned())
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
    let mut restore: Vec<Value> = Vec::new();
    for event in store.audit("stray_files_moved", 50)? {
        for path in event["paths"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            let stored = dir.join("stray-files").join(path);
            if !stored.is_file()
                || restore.iter().any(|r| r["path"] == path)
                || !scopes(&plan).iter().any(|s| within(path, s))
            {
                continue;
            }
            let destination = root.join(path);
            let action = if !destination.exists() {
                "restore"
            } else if fs::read(&destination).ok() == fs::read(&stored).ok() {
                "already_present"
            } else {
                "conflict"
            };
            restore.push(json!({"path": path, "action": action}));
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

/// The exact change an approval would apply, with the digest that approval must quote.
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

/// Applies the reviewed change, restores what is safe to restore, and readies the run.
pub fn recovery_apply(
    dir: &Path,
    task: &str,
    paths: &[String],
    test: Option<Vec<String>>,
    approved: &str,
) -> Result<(), String> {
    // The room polls the run while this worker starts; wait out a brief hold instead of failing the job.
    let _lock = (0..40)
        .find_map(|_| match Lock::acquire(dir) {
            Err(e) if e.contains("lease is held") => {
                std::thread::sleep(std::time::Duration::from_millis(50));
                None
            }
            other => Some(other),
        })
        .unwrap_or_else(|| Lock::acquire(dir))?;
    let current = recovery_preview(dir, task, paths, test.clone())?;
    if current["digest"] != approved {
        return Err("the proposal changed or was not reviewed; preview it again".into());
    }
    let Proposal { plan, .. } = proposal(dir, task, paths, test.as_deref())?;
    let mut report = load(dir)?;
    let root = Repository::open(&report.repo)?.root;
    let mut restored = Vec::new();
    for item in current["restore"].as_array().into_iter().flatten() {
        let path = item["path"].as_str().unwrap_or_default();
        if item["action"] == "restore" {
            let to = root.join(path);
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(dir.join("stray-files").join(path), &to).map_err(|e| e.to_string())?;
            restored.push(path.to_owned());
        }
    }
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record(
        "scope_recovery_approved",
        &json!({"proposal": current, "restored": restored, "previous_plan_digest": digest(&serde_json::to_vec(&report.plan).map_err(|e| e.to_string())?)}),
        "Operator approved a reviewed scope change; finished work is reused and integration verification reruns",
    )?;
    // DD-174: later tasks and the operator learn which paths the task really owns.
    let cause = report.error.clone().unwrap_or_default();
    let response = format!("Owned paths of this task were widened to {paths:?} after review");
    super::step_lessons::note(dir, &mut store, task, &cause, &response)?;
    // DD-180: the operator approved this scope, so it becomes the charter's baseline.
    let mut updated = super::guardrail::charter(&plan);
    updated["status"] = "updated".into();
    store.record(
        "charter",
        &updated,
        "Charter updated: the operator approved a scope change",
    )?;
    report.plan = plan;
    report.candidate = None;
    report.diff_hash = None;
    report.tests.clear();
    report.error = None;
    // Stopped, so the existing explicit Resume continues from the finished work.
    report.state = "stopped".into();
    store.record(
        "approval",
        &Value::Null,
        "Scope recovery revoked prior integration approval",
    )?;
    save(&mut store, dir, &report)
}

/// The broker command line the Agent Room runs to apply an approved change.
pub fn scope_recovery_args(
    task: &str,
    paths: &[String],
    test: Option<&[String]>,
    digest: &str,
) -> Result<Vec<String>, String> {
    let mut args = vec![
        "scope-recovery".into(),
        task.into(),
        digest.into(),
        serde_json::to_string(paths).map_err(|e| e.to_string())?,
    ];
    if let Some(test) = test {
        args.push(serde_json::to_string(test).map_err(|e| e.to_string())?);
    }
    Ok(args)
}
