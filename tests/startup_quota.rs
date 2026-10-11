#![cfg(unix)]
use agentconductor::{quota, state::Store, ui::setup::Startup, workflow};
use serde_json::json;
#[path = "support/workflow.rs"]
mod support;
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
