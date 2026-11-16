//! GitHub #1014: the tool-context verifier must not repeat Git work. A plain folder opens its
//! external history without a certain-to-fail `rev-parse`, and a shared workspace reuses the
//! target repository instead of reopening it and comparing its common dir with itself.
#![cfg(unix)]
use agentconductor::{folder, gates::ToolRequest, workflow};
use serde_json::json;
use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
};
#[path = "support/workflow.rs"]
mod support;

fn calls(log: &Path, needle: &str) -> usize {
    fs::read_to_string(log)
        .unwrap_or_default()
        .lines()
        .filter(|l| l.contains(needle))
        .count()
}

#[tokio::test]
async fn the_verifier_skips_redundant_git_processes() {
    // A logging `git` in front of the real one; one test only, because it changes PATH.
    let shim = std::env::temp_dir().join(format!("ac-git-shim-{}", std::process::id()));
    fs::create_dir_all(&shim).unwrap();
    let log = shim.join("calls.log");
    let real = String::from_utf8(Command::new("which").arg("git").output().unwrap().stdout)
        .unwrap()
        .trim()
        .to_string();
    let script = shim.join("git");
    fs::write(
        &script,
        format!(
            "#!/bin/sh\necho \"$*\" >> '{}'\nexec '{real}' \"$@\"\n",
            log.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    let path = format!("{}:{}", shim.display(), std::env::var("PATH").unwrap());
    // SAFETY: the only test in this binary, set before any thread reads the environment.
    unsafe { std::env::set_var("PATH", &path) };

    // A plain folder: one toplevel check, made against its external history.
    let (root, _) = support::setup("shared");
    let project = root.join("repo");
    fs::remove_dir_all(project.join(".git")).unwrap();
    let planned = folder::repository(&project, &root.join("history")).unwrap();
    folder::activate(&planned, &project).unwrap();
    let _ = fs::remove_file(&log);
    let git = agentconductor::git::Repository::open(&project).unwrap();
    assert!(git.is_external());
    assert_eq!(calls(&log, "--show-toplevel"), 1);
    fs::remove_dir_all(root).unwrap();

    // A shared workspace is the target folder: opened once, no common-dir comparison.
    let (root, plan) = support::setup("shared");
    let repo = root.join("repo");
    let dir = root.join("run");
    workflow::run(plan, &repo, &dir, 5).await.unwrap();
    let request = ToolRequest {
        tool: "Bash".into(),
        input: json!({"command":"curl -sS -o report https://example.invalid"}),
        reason: "Command approval".into(),
    };
    let _ = fs::remove_file(&log);
    let mut child = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .args(["tool-context", dir.to_str().unwrap(), "one.txt"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&request).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(calls(&log, "--show-toplevel"), 1);
    assert_eq!(calls(&log, "--git-common-dir"), 0);
    assert_eq!(calls(&log, "rev-parse HEAD"), 1);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(shim).unwrap();
}
