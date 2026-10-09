#![cfg(unix)]
use agentconductor::{
    gates::{Context, ToolRequest},
    workflow,
};
use serde_json::json;
use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};
#[path = "support/workflow.rs"]
mod support;

fn context(dir: &std::path::Path, task: &str, request: &ToolRequest) -> Result<Context, String> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .args(["tool-context", dir.to_str().unwrap(), task])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(request).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }
    serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())
}

#[tokio::test]
async fn compiled_verifier_derives_binding_from_stored_plan_session_and_current_git_state() {
    for mode in ["shared", "worktree"] {
        let (root, plan) = support::setup(mode);
        let repo = root.join("repo");
        let dir = root.join("run");
        let report = workflow::run(plan, &repo, &dir, 5).await.unwrap();
        let request = ToolRequest {
            tool: "Bash".into(),
            input: json!({"command":"cargo test"}),
            reason: "Command approval".into(),
        };
        let first = context(&dir, "one.txt", &request).unwrap();
        let bounded = workflow::verify_tool_context(
            &dir,
            "one.txt",
            &request,
            std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
        )
        .await
        .unwrap();
        assert_eq!(first, bounded);
        assert_eq!(first.task, "one.txt");
        assert_eq!(
            Some(first.session.as_str()),
            report.tasks[0].session_id.as_deref()
        );
        assert_eq!(
            first.workspace,
            report.tasks[0]
                .cwd
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
        );
        assert_eq!(
            first.plan_sha256,
            workflow::digest(&serde_json::to_vec(&report.plan).unwrap())
        );
        fs::write(report.tasks[0].cwd.join("one.txt"), "changed after review").unwrap();
        let second = context(&dir, "one.txt", &request).unwrap();
        assert_ne!(first.workspace_sha256, second.workspace_sha256);
        assert_eq!(first.plan_sha256, second.plan_sha256);
        assert!(context(&dir, "invented", &request).is_err());
        assert!(
            context(
                &dir,
                "one.txt",
                &ToolRequest {
                    tool: "Write".into(),
                    input: json!({"file_path":"two.txt","content":"cross-task"}),
                    reason: "Override".into()
                }
            )
            .is_err()
        );
        fs::remove_dir_all(root).unwrap();
    }
}

#[tokio::test]
async fn ignored_sensitive_files_affect_the_binding_without_becoming_git_objects() {
    let (root, plan) = support::setup("shared");
    let repo = root.join("repo");
    let dir = root.join("run");
    fs::write(repo.join(".git/info/exclude"), ".env\n").unwrap();
    workflow::run(plan, &repo, &dir, 5).await.unwrap();
    let request = ToolRequest {
        tool: "Read".into(),
        input: json!({"file_path":".env"}),
        reason: "Sensitive read".into(),
    };
    fs::write(repo.join(".env"), "synthetic-value-one").unwrap();
    let first = context(&dir, "one.txt", &request).unwrap();
    fs::write(repo.join(".env"), "synthetic-value-two").unwrap();
    let second = context(&dir, "one.txt", &request).unwrap();
    assert_ne!(first.workspace_sha256, second.workspace_sha256);
    let objects = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["rev-list", "--objects", "--all"])
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&objects.stdout).contains(".env"));
    fs::remove_dir_all(root).unwrap();
}
