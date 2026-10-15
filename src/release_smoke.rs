use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

pub struct ValidatedPackage {
    pub binary: PathBuf,
    pub manifest: Value,
}

pub fn validate_package(package: &Path) -> Result<ValidatedPackage, String> {
    let package = package
        .canonicalize()
        .map_err(|error| format!("resolve package directory: {error}"))?;
    if !package.is_dir() {
        return Err("package must be a directory".into());
    }
    let manifest: Value = serde_json::from_slice(
        &fs::read(package.join("manifest.json"))
            .map_err(|error| format!("read manifest: {error}"))?,
    )
    .map_err(|error| format!("parse package manifest: {error}"))?;
    let name = manifest["binary"]
        .as_str()
        .ok_or_else(|| "manifest binary name is missing".to_owned())?;
    if !matches!(name, "agentconductor" | "agentconductor.exe") {
        return Err("invalid packaged binary path".into());
    }
    let binary = package.join(name);
    let metadata =
        fs::symlink_metadata(&binary).map_err(|error| format!("read packaged binary: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("packaged binary must be a regular file".into());
    }
    let digest = sha256_file(&binary)?;
    if manifest["binary_sha256"].as_str() != Some(&digest) {
        return Err("packaged binary checksum mismatch".into());
    }
    let commit = manifest["source_commit"]
        .as_str()
        .ok_or_else(|| "exact packaged source commit required".to_owned())?;
    if commit.len() != 40
        || !commit
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("exact packaged source commit required".into());
    }
    let sums = fs::read_to_string(package.join("SHA256SUMS"))
        .map_err(|error| format!("read package checksum file: {error}"))?;
    if sums != format!("{digest}  {name}\n") {
        return Err("package checksum file mismatch".into());
    }
    Ok(ValidatedPackage { binary, manifest })
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| format!("open packaged binary: {error}"))?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let length = file
            .read(&mut buffer)
            .map_err(|error| format!("read packaged binary: {error}"))?;
        if length == 0 {
            break;
        }
        digest.update(&buffer[..length]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

pub async fn run_smoke(package: &Path, evidence: &Path) -> Result<Value, String> {
    let validated = validate_package(package)?;
    fs::create_dir_all(evidence).map_err(|error| error.to_string())?;
    let evidence = evidence.canonicalize().map_err(|error| error.to_string())?;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "ac-release-smoke-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).map_err(|error| error.to_string())?;
    let worker = root.join(if cfg!(windows) {
        "fixture.exe"
    } else {
        "fixture"
    });
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/release_worker.rs");
    let logs = evidence.join("commands");
    checked(
        &[
            "rustc".into(),
            source.to_string_lossy().into_owned(),
            "-o".into(),
            worker.to_string_lossy().into_owned(),
        ],
        &root,
        &logs,
        None,
        false,
    )
    .await?;
    checked(
        &[
            validated.binary.to_string_lossy().into_owned(),
            "--help".into(),
        ],
        &root,
        &logs,
        Some((&evidence, "help")),
        false,
    )
    .await?;

    let mut cases = Vec::new();
    for mode in ["shared", "worktree"] {
        for folder in [false, true] {
            cases.push(
                workflow_smoke(&validated.binary, &worker, &root, &logs, mode, folder).await?,
            );
        }
    }
    let result = json!({"manifest":validated.manifest,"cases":cases,"exit_code":0});
    let mut bytes = serde_json::to_vec_pretty(&result).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    fs::write(evidence.join("smoke.json"), bytes).map_err(|error| error.to_string())?;
    fs::remove_dir_all(&root).map_err(|error| format!("remove smoke workspace: {error}"))?;
    Ok(result)
}

async fn workflow_smoke(
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

fn command(binary: &Path, args: &[&str]) -> Vec<String> {
    std::iter::once(binary.to_string_lossy().into_owned())
        .chain(args.iter().map(|arg| arg.to_string()))
        .collect()
}

fn prefixed(program: &str, args: &[&str]) -> Vec<String> {
    std::iter::once(program.to_owned())
        .chain(args.iter().map(|arg| arg.to_string()))
        .collect()
}

async fn checked(
    command: &[String],
    cwd: &Path,
    logs: &Path,
    evidence: Option<(&Path, &str)>,
    denied: bool,
) -> Result<String, String> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    fs::create_dir_all(logs).map_err(|error| error.to_string())?;
    let log_dir = logs.join(format!("command-{}", NEXT.fetch_add(1, Ordering::Relaxed)));
    let result = crate::process::run(crate::process::ProcessSpec {
        executable: PathBuf::from(&command[0]),
        args: command.iter().skip(1).cloned().collect(),
        cwd: cwd.to_owned(),
        input: Vec::new(),
        log_dir,
        timeout: Duration::from_secs(120),
        signal_grace: Duration::from_secs(2),
        environment: BTreeMap::new(),
    })
    .await?;
    let stdout = fs::read(&result.stdout).map_err(|error| error.to_string())?;
    let stderr = fs::read(&result.stderr).map_err(|error| error.to_string())?;
    let stdout = String::from_utf8_lossy(&stdout).into_owned();
    let stderr = String::from_utf8_lossy(&stderr).into_owned();
    if let Some((directory, name)) = evidence {
        fs::write(directory.join(format!("{name}.stdout")), &stdout)
            .map_err(|error| error.to_string())?;
        fs::write(directory.join(format!("{name}.stderr")), &stderr)
            .map_err(|error| error.to_string())?;
    }
    if denied {
        if result.exit_code == Some(0) || !stderr.to_lowercase().contains("approval") {
            return Err(format!("unapproved merge was not refused: {stderr}"));
        }
    } else if result.exit_code != Some(0) || result.timed_out {
        let detail = if command.get(1).map(String::as_str) == Some("run") {
            let status = crate::process::run(crate::process::ProcessSpec {
                executable: PathBuf::from(&command[0]),
                args: vec!["status".into(), command[4].clone()],
                cwd: cwd.to_owned(),
                input: Vec::new(),
                log_dir: logs.join(format!("status-{}", NEXT.fetch_add(1, Ordering::Relaxed))),
                timeout: Duration::from_secs(60),
                signal_grace: Duration::from_secs(2),
                environment: BTreeMap::new(),
            })
            .await?;
            format!(
                "\nRun status:\n{}{}",
                String::from_utf8_lossy(&fs::read(status.stdout).unwrap_or_default()),
                String::from_utf8_lossy(&fs::read(status.stderr).unwrap_or_default())
            )
        } else {
            String::new()
        };
        return Err(format!(
            "command failed: {}\n{}{}",
            command.join(" "),
            stderr,
            detail
        ));
    }
    Ok(stdout)
}
