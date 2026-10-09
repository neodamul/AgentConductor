#![cfg(unix)]
use agentconductor::leases::SessionLease;
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn workflow_refuses_to_spawn_a_resume_while_native_session_is_leased_elsewhere() {
    use agentconductor::workflow;
    let (root, mut plan) = support::setup("shared");
    let mut profile = serde_json::to_value(&plan.agents[0]).unwrap();
    profile["catalog"] = serde_json::json!({"models":["local/one"],"source":"test","queried_at":1,"cli_version":"fixture"});
    profile["args"] = serde_json::json!(["task", "{model}", "{session_id}"]);
    plan.agents[0] = serde_json::from_value(profile).unwrap();
    let dir = root.join("run");
    let initial = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    let agent = &initial.plan.agents[0];
    let key = serde_json::to_string(&(
        agent.parser,
        agent.quota_key(),
        initial.tasks[0].session_id.as_deref().unwrap(),
    ))
    .unwrap();
    let lease = SessionLease::acquire(&key, &initial.tasks[0].artifacts.join("process/owner.json"))
        .unwrap();
    workflow::set_model(&dir, "one.txt", "local/one").unwrap();
    workflow::stop(&dir, "inspect session ownership").unwrap();
    let report = workflow::resume(&dir, 10).await.unwrap();
    assert_eq!(report.tasks[0].status, "failed");
    assert!(
        report.tasks[0]
            .error
            .as_ref()
            .unwrap()
            .contains("lease is held")
    );
    assert!(!report.tasks[0].artifacts.join("process").exists());
    assert!(workflow::merge(&dir).is_err());
    drop(lease);
    std::fs::remove_dir_all(root).unwrap();
}

fn setup(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ac-session-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
fn record(path: &std::path::Path, pid: Option<u32>) {
    let owner = serde_json::json!({"coordinator_pid":std::process::id(),"pid":pid,"process_group":pid,
        "phase":if pid.is_some(){"running"}else{"not_spawned"},"exit_code":null,"stopped":false,"timed_out":false});
    std::fs::write(path, serde_json::to_vec(&owner).unwrap()).unwrap();
}
#[test]
fn same_native_session_is_exclusive_across_profiles_and_workspaces() {
    let dir = setup("exclusive");
    let path = dir.join("owner.json");
    record(&path, None);
    let key = format!("codex/account/exclusive-{}", std::process::id());
    let lease = SessionLease::acquire(&key, &path).unwrap();
    assert!(SessionLease::acquire(&key, &dir.join("other.json")).is_err());
    drop(lease);
    let lease = SessionLease::acquire(&key, &path).unwrap();
    drop(lease);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn live_or_missing_prior_process_record_blocks_reuse_after_lease_release() {
    let dir = setup("crashed");
    let path = dir.join("owner.json");
    record(&path, Some(std::process::id()));
    let key = format!("claude/account/crashed-{}", std::process::id());
    drop(SessionLease::acquire(&key, &path).unwrap());
    assert!(SessionLease::acquire(&key, &dir.join("other.json")).is_err());
    std::fs::remove_file(&path).unwrap();
    assert!(SessionLease::acquire(&key, &dir.join("other.json")).is_err());
    record(&path, Some(i32::MAX as u32));
    let lease = SessionLease::acquire(&key, &path).unwrap();
    drop(lease);
    std::fs::remove_dir_all(dir).unwrap();
}
