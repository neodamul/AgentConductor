//! R-45 / DD-53 / QAS-38: planning shows coordinator, elapsed time, ready and failure states.
use agentconductor::ui::setup::Startup;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

fn snapshot(startup: &Startup) -> serde_json::Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(startup: &Startup, mut packet: serde_json::Value) -> Result<serde_json::Value, String> {
    packet["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(packet.to_string().as_bytes())
}

#[test]
fn job_snapshot_carries_start_time_and_log_after_reopen() {
    let (root, plan) = support::setup("shared");
    let mut profiles = plan.agents;
    let mut planner = profiles[0].clone();
    planner.id = "planner".into();
    planner.args = vec!["plan".into()];
    planner.permission_mode = "read_only".into();
    profiles.push(planner);
    let dir = root.join("desktop");
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let startup = Startup::open(&dir, exe).unwrap();
    mutate(&startup, serde_json::json!({"kind":"save","config":{"profiles":profiles,"repository":root.join("repo"),"goal":"Three files","coordinator":"planner","workspace_mode":"shared"}})).unwrap();
    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    mutate(&startup, serde_json::json!({"kind":"prepare"})).unwrap();
    let job = snapshot(&startup)["job"].clone();
    // DD-91: the coordinator's own output activity, not only elapsed time.
    assert!(job["output"]["bytes"].is_u64(), "{job}");
    let started = job["started_ms"].as_u64().expect("started_ms");
    assert!(started >= before.saturating_sub(1000), "{job}");
    assert!(
        Path::new(job["log"].as_str().expect("log")).is_dir(),
        "{job}"
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while snapshot(&startup)["job"]["state"] == "running" {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    drop(startup);
    let reopened = Startup::open(&dir, exe).unwrap();
    let job = snapshot(&reopened)["job"].clone();
    assert_eq!(job["state"], "completed");
    assert_eq!(
        job["started_ms"].as_u64(),
        Some(started),
        "reopened window keeps the true start"
    );
    drop(reopened);
    std::fs::remove_dir_all(root).unwrap();
}
