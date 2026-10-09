use agentconductor::plan::Plan;
use std::{path::PathBuf, process::Command, sync::OnceLock};
pub fn fixture() -> PathBuf {
    static MOCK: OnceLock<PathBuf> = OnceLock::new();
    MOCK.get_or_init(|| {
        let path = std::env::temp_dir().join(format!("ac-workflow-mock-{}", std::process::id()));
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
pub fn setup(mode: &str) -> (PathBuf, Plan) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "ac-flow-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
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
