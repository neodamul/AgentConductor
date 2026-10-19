use agentconductor::plan::Plan;
use std::{path::PathBuf, process::Command, sync::OnceLock};

fn cargo_tmp() -> PathBuf {
    let dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tmp");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn fixture() -> PathBuf {
    static MOCK: OnceLock<PathBuf> = OnceLock::new();
    MOCK.get_or_init(|| {
        let mut path = cargo_tmp().join(format!("ac-workflow-mock-{}", std::process::id()));
        if cfg!(windows) {
            path.set_extension("exe");
        }
        assert!(
            Command::new("rustc")
                .args(["tests/support/mock_agent.rs", "-o"])
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        path
    })
    .clone()
}
#[allow(dead_code)]
pub fn timeout_once_fixture() -> PathBuf {
    static MOCK: OnceLock<PathBuf> = OnceLock::new();
    MOCK.get_or_init(|| {
        let mut path =
            std::env::temp_dir().join(format!("ac-timeout-agent-{}", std::process::id()));
        if cfg!(windows) {
            path.set_extension("exe");
        }
        assert!(
            Command::new("rustc")
                .args(["tests/support/timeout_once_agent.rs", "-o"])
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        path
    })
    .clone()
}
pub fn setup(mode: &str) -> (PathBuf, Plan) {
    // PIDs can be reused after an interrupted test; retain failed evidence and
    // allocate a fresh directory instead of colliding with a previous run.
    let root = cargo_tmp().join(format!(
        "ac-flow-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir_all(root.join("repo")).unwrap();
    for args in [
        vec!["init", "-b", "main"],
        vec!["config", "user.name", "Test"],
        vec!["config", "user.email", "test@example.invalid"],
    ] {
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(root.join("repo"))
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    std::fs::write(root.join("repo/base.txt"), "base\n").unwrap();
    for args in [vec!["add", "."], vec!["commit", "-m", "base"]] {
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(root.join("repo"))
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    let plan = serde_json::json!({"goal":"implement three independent files","workspace_mode":mode,
        "agents":[{"id":"mock","executable":fixture(),"args":["task"],"permission_mode":"external_sandbox"}],
        "tasks":(["one.txt","two.txt","three.txt"].map(|path|serde_json::json!({"id":path,"agent":"mock","prompt":path,"paths":[path],"test_command":[fixture(),"test"]})))});
    (root, serde_json::from_value(plan).unwrap())
}
