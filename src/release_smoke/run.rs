use super::{validate::validate_package, workflow::workflow_smoke};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

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

pub(super) fn command(binary: &Path, args: &[&str]) -> Vec<String> {
    std::iter::once(binary.to_string_lossy().into_owned())
        .chain(args.iter().map(|arg| arg.to_string()))
        .collect()
}

pub(super) fn prefixed(program: &str, args: &[&str]) -> Vec<String> {
    std::iter::once(program.to_owned())
        .chain(args.iter().map(|arg| arg.to_string()))
        .collect()
}

pub(super) async fn checked(
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
