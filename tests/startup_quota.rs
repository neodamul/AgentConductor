#![cfg(unix)]
use agentconductor::{quota, state::Store, ui::setup::Startup, workflow};
use serde_json::json;
#[path = "support/workflow.rs"]
mod support;

/// R-66: workspaces start empty, so tests register the provider templates first.
fn add_template_agents(startup: &Startup) {
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let packet =
        json!({"kind":"save_profiles","revision":state["revision"],"profiles":state["templates"]});
    startup.dispatch(packet.to_string().as_bytes()).unwrap();
}
#[tokio::test]
async fn workspace_badges_read_retained_quota_and_invalidate_changed_profiles() {
    let (root, plan) = support::setup("shared");
    let workspace = root.join("desktop");
    std::fs::create_dir(&workspace).unwrap();
    let run = workspace.join("run");
    workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let startup = Startup::open(&workspace, exe).unwrap();
    add_template_agents(&startup);
    let before = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(before["quotas"]["codex"]["status"], "unavailable");
    let profile = serde_json::from_value(before["config"]["profiles"][1].clone()).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let mut store = Store::open(run.join("state.sqlite")).unwrap();
    quota::persist(
        &mut store,
        &profile,
        quota::parse_codex(
            &json!({"rateLimits":{"primary":{"usedPercent":27,"resetsAt":now+1000}}}),
            now,
        ),
    )
    .unwrap();
    drop(store);
    drop(startup);
    let settings = workspace.join("startup.json");
    let mut saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&settings).unwrap()).unwrap();
    saved["last_run"] = json!(run);
    std::fs::write(&settings, serde_json::to_vec(&saved).unwrap()).unwrap();
    let startup = Startup::open(&workspace, exe).unwrap();
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(
        state["quotas"]["codex"]["windows"][0]["remaining_percent"],
        73.0
    );
    let mut profiles = state["config"]["profiles"].clone();
    profiles[1]["quota_refresh_minutes"] = 10.into();
    startup
        .dispatch(
            json!({"kind":"save_profiles","revision":state["revision"],"profiles":profiles})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(
        state["quotas"]["codex"]["windows"][0]["remaining_percent"],
        73.0
    );
    assert_eq!(
        quota::stored_for_run(
            &Store::read_only_snapshot(run.join("state.sqlite")).unwrap(),
            &profile,
            now,
            &run
        )
        .unwrap()["refresh_minutes"],
        10
    );
    let mut config = state["config"].clone();
    config["profiles"][1]["executable"] = "changed-login".into();
    startup
        .dispatch(
            json!({"kind":"save","revision":state["revision"],"config":config})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    assert_eq!(
        startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["quotas"]["codex"]["status"],
        "unavailable"
    );
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn fresh_workspace_queries_codex_without_any_execution_run() {
    let root = std::env::temp_dir().join(format!(
        "ac-startup-quota-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let executable = root.join("quota-fixture");
    assert!(
        std::process::Command::new("rustc")
            .args(["tests/support/quota_agent.rs", "-o"])
            .arg(&executable)
            .status()
            .unwrap()
            .success()
    );
    let startup = Startup::open(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    add_template_agents(&startup);
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let mut config = state["config"].clone();
    config["profiles"][1]["executable"] = json!(executable);
    config["profiles"][1]["permission_mode"] = "unverified".into();
    let release = root.join("quota-release");
    config["profiles"][1]["environment"]["QUOTA_TEST_RELEASE"] = json!(release);
    startup
        .dispatch(
            json!({"kind":"save","revision":state["revision"],"config":config})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    startup
        .dispatch(
            json!({"kind":"quota","agent":"codex","revision":state["revision"]})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let waiting = release.with_extension("waiting");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !waiting.exists() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    // The provider is still waiting; dispatch and snapshot both return before collection.
    let running = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(running["job"]["state"], "running");
    assert_eq!(running["quotas"]["codex"]["status"], "unavailable");
    let duplicate = startup.dispatch(
        json!({"kind":"quota","agent":"codex","revision":running["revision"]})
            .to_string()
            .as_bytes(),
    );
    assert!(duplicate.is_err());
    std::fs::write(&release, b"release").unwrap();
    loop {
        let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
        if state["job"]["state"] != "running" {
            assert_eq!(
                state["quotas"]["codex"]["windows"][0]["remaining_percent"],
                73.0
            );
            assert_eq!(state["run_available"], false);
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn room_uses_only_its_bound_workspace_cache() {
    let root = std::env::temp_dir().join(format!(
        "ac-quota-binding-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir_all(root.join("run")).unwrap();
    let run = root.join("run");
    let profile = serde_json::from_value(
        json!({"id":"codex","parser":"codex","executable":"codex","args":[]}),
    )
    .unwrap();
    let store = Store::open(run.join("state.sqlite")).unwrap();
    let mut cache = Store::open(root.join("quota.sqlite")).unwrap();
    quota::persist(
        &mut cache,
        &profile,
        quota::parse_codex(&json!({"rateLimits":{"primary":{"usedPercent":27}}}), 1000),
    )
    .unwrap();
    std::fs::write(
        root.join("startup.json"),
        serde_json::to_vec(&json!({"last_run":root.join("other")})).unwrap(),
    )
    .unwrap();
    assert_eq!(
        quota::stored_for_run(&store, &profile, 1001, &run).unwrap()["status"],
        "unavailable"
    );
    std::fs::write(
        root.join("startup.json"),
        serde_json::to_vec(&json!({"last_run":run})).unwrap(),
    )
    .unwrap();
    assert_eq!(
        quota::stored_for_run(&store, &profile, 1001, &run).unwrap()["windows"][0]["remaining_percent"],
        73.0
    );
    drop(cache);
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn workspace_snapshot_tolerates_cache_before_schema_initialization() {
    let (root, _) = support::setup("shared");
    let workspace = root.join("desktop");
    let startup = Startup::open(
        &workspace,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    add_template_agents(&startup);
    drop(rusqlite::Connection::open(workspace.join("quota.sqlite")).unwrap());
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(state["quotas"]["codex"]["status"], "unavailable");
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
