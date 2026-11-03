use super::run::{checked, command, prefixed};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub(super) async fn workflow_smoke(
    binary: &Path,
    worker: &Path,
    root: &Path,
    logs: &Path,
    mode: &str,
    folder: bool,
) -> Result<Value, String> {
    let directory = root.join(format!("{}{}", if folder { "folder-" } else { "" }, mode));
    let repo = directory.join("repo");
    let evidence = directory.join("evidence");
    let run = directory.join("run");
    fs::create_dir_all(&repo).map_err(|error| error.to_string())?;
    fs::create_dir(&evidence).map_err(|error| error.to_string())?;
    fs::write(repo.join("base.txt"), "base\n").map_err(|error| error.to_string())?;

    let base = if folder {
        None
    } else {
        for args in [
            vec!["init", "-b", "main"],
            vec!["config", "user.name", "Release fixture"],
            vec!["config", "user.email", "release@example.invalid"],
            vec!["add", "base.txt"],
            vec!["commit", "-m", "base"],
        ] {
            checked(&prefixed("git", &args), &repo, logs, None, false).await?;
        }
        Some(
            checked(
                &prefixed("git", &["rev-parse", "HEAD"]),
                &repo,
                logs,
                None,
                false,
            )
            .await?
            .trim()
            .to_owned(),
        )
    };

    let paths = ["one.txt", "two.txt", "three.txt"];
    let tasks: Vec<_> = paths
        .iter()
        .map(|path| {
            json!({
                "id":path,
                "agent":"fixture",
                "prompt":path,
                "paths":[path],
                "test_command":[worker.to_string_lossy(),"test"]
            })
        })
        .collect();
    let plan = json!({
        "goal":"Verify packaged native three-worker orchestration",
        "workspace_mode":mode,
        "agents":[{
            "id":"fixture",
            "executable":worker,
            "args":["task"],
            "permission_mode":"external_sandbox"
        }],
        "tasks":tasks
    });
    let plan_path = directory.join("plan.json");
    fs::write(
        &plan_path,
        serde_json::to_vec(&plan).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let validated = checked(
        &command(binary, &["validate", &plan_path.to_string_lossy()]),
        &repo,
        logs,
        Some((&evidence, "validate")),
        false,
    )
    .await?;
    let consent = serde_json::from_str::<Value>(&validated)
        .map_err(|error| error.to_string())?["plan_digest"]
        .as_str()
        .ok_or_else(|| "plan validation omitted digest".to_owned())?
        .to_owned();
    let report_text = checked(
        &command(
            binary,
            &[
                "run",
                &plan_path.to_string_lossy(),
                &repo.to_string_lossy(),
                &run.to_string_lossy(),
                &consent,
            ],
        ),
        &repo,
        logs,
        Some((&evidence, "run")),
        false,
    )
    .await?;
    let report: Value = serde_json::from_str(&report_text).map_err(|error| error.to_string())?;
    let task_rows = report["tasks"]
        .as_array()
        .ok_or_else(|| "workflow report omitted tasks".to_owned())?;
    let tests = report["tests"]
        .as_array()
        .ok_or_else(|| "workflow report omitted tests".to_owned())?;
    if report["state"] != "waiting_approval"
        || task_rows.iter().any(|task| task["status"] != "completed")
        || tests.is_empty()
        || tests.iter().any(|test| test["exit_code"] != 0)
    {
        return Err("packaged workflow failed or did not wait for approval".into());
    }
    let ranges: Vec<(u64, u64)> = task_rows
        .iter()
        .map(|task| {
            Ok((
                task["started_ms"]
                    .as_u64()
                    .ok_or_else(|| "task start time missing".to_owned())?,
                task["finished_ms"]
                    .as_u64()
                    .ok_or_else(|| "task finish time missing".to_owned())?,
            ))
        })
        .collect::<Result<_, String>>()?;
    if ranges.iter().map(|range| range.0).max().unwrap_or(0)
        >= ranges.iter().map(|range| range.1).min().unwrap_or(0)
    {
        return Err("three workers did not overlap".into());
    }
    let diff = checked(
        &command(binary, &["diff", &run.to_string_lossy()]),
        &repo,
        logs,
        Some((&evidence, "diff")),
        false,
    )
    .await?;
    if paths.iter().any(|path| !diff.contains(path)) {
        return Err("diff omitted owned changes".into());
    }
    if folder && repo.join(".git").exists() {
        return Err("original folder was initialized before approval".into());
    }
    if !folder
        && checked(
            &prefixed("git", &["rev-parse", "HEAD"]),
            &repo,
            logs,
            None,
            false,
        )
        .await?
        .trim()
            != base.as_deref().unwrap_or_default()
    {
        return Err("target branch changed before approval".into());
    }
    checked(
        &command(binary, &["merge", &run.to_string_lossy()]),
        &repo,
        logs,
        Some((&evidence, "unapproved-merge")),
        true,
    )
    .await?;
    let candidate = report["candidate"]["commit"]
        .as_str()
        .ok_or_else(|| "workflow report omitted candidate commit".to_owned())?;
    checked(
        &command(binary, &["approve", &run.to_string_lossy(), candidate]),
        &repo,
        logs,
        Some((&evidence, "approve")),
        false,
    )
    .await?;
    checked(
        &command(binary, &["merge", &run.to_string_lossy()]),
        &repo,
        logs,
        Some((&evidence, "merge")),
        false,
    )
    .await?;
    if folder && repo.join(".git").exists() {
        return Err("original folder was initialized".into());
    }
    if !folder
        && checked(
            &prefixed("git", &["rev-parse", "HEAD"]),
            &repo,
            logs,
            None,
            false,
        )
        .await?
        .trim()
            != candidate
    {
        return Err("approved candidate was not merged".into());
    }
    if !folder
        && !checked(
            &prefixed("git", &["status", "--porcelain"]),
            &repo,
            logs,
            None,
            false,
        )
        .await?
        .trim()
        .is_empty()
    {
        return Err("merged repository is dirty".into());
    }
    for path in paths {
        let contents = fs::read_to_string(repo.join(path)).map_err(|error| error.to_string())?;
        if contents.replace("\r\n", "\n") != "implemented\n" {
            return Err("merged scoped content mismatch".into());
        }
    }
    Ok(json!({
        "mode":mode,
        "source_mode":if folder { "folder" } else { "git" },
        "base":base,
        "candidate":candidate,
        "parallel_ranges":ranges,
        "tasks":3,
        "tests":tests.len(),
        "unapproved_merge_refused":true,
        "approved_merge_verified":true
    }))
}
