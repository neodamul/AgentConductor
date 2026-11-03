use agentconductor::{state::Store, ui::Room, workflow};
use serde_json::json;
use std::path::Path;

fn snapshot_output(stderr: &str, stdout: &str) -> String {
    let root = std::env::temp_dir().join(format!(
        "ac-ui-failure-report-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let repo = root.join("repo");
    std::fs::create_dir_all(repo.join(".git")).unwrap();
    let run = root.join("run");
    std::fs::create_dir_all(&run).unwrap();
    let stdout_path = run.join("integration.stdout");
    let stderr_path = run.join("integration.stderr");
    std::fs::write(&stdout_path, stdout).unwrap();
    std::fs::write(&stderr_path, stderr).unwrap();
    let report = json!({
        "schema_version":1,"state":"failed","repo":repo,"base":"base","branch":"main",
        "plan":{"goal":"Run tests","agents":[],"tasks":[],"workspace_mode":"shared"},
        "tasks":[],"candidate":{"commit":"candidate","tree":"tree","changed":[]},
        "tests":[{"task":"integration","exit_code":1,"timed_out":false,
            "stdout":stdout_path,"stderr":stderr_path,
            "stdout_hash":workflow::digest(stdout.as_bytes()),
            "stderr_hash":workflow::digest(stderr.as_bytes())}],
        "diff_hash":workflow::digest(b""),"error":"integration tests failed",
        "verification_cwd":repo
    });
    Store::open(run.join("state.sqlite"))
        .unwrap()
        .record("workflow", &report, "failure-output fixture")
        .unwrap();
    std::fs::write(run.join("report.json"), report.to_string()).unwrap();
    let room = Room::open(&run, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let output = snapshot["report"]["tests"][0]["output"]
        .as_str()
        .unwrap()
        .to_owned();
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
    output
}

#[test]
fn failed_test_report_keeps_stdout_error_when_stderr_has_only_warning() {
    let error = "Error: Cannot find module tests\\\\service";
    let warning = "[UNDICI-EHPA] EnvHttpProxyAgent is experimental";
    let output = snapshot_output(warning, error);
    assert_eq!(output, format!("stderr:\n{warning}\n\nstdout:\n{error}"));
}

#[test]
fn single_stream_failure_output_is_returned_unchanged() {
    assert_eq!(snapshot_output("stderr error\n", ""), "stderr error\n");
    assert_eq!(snapshot_output("", "stdout error\n"), "stdout error\n");
}
