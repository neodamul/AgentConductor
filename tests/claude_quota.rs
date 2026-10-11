use agentconductor::{plan::AgentProfile, protocol::Provider, quota, state::Store};
use serde_json::json;
fn profile() -> AgentProfile {
    serde_json::from_value(json!({"id":"claude","parser":"claude","executable":"must-not-launch","args":["-p","--output-format","stream-json"]})).unwrap()
}
fn event(utilization: serde_json::Value) -> serde_json::Value {
    json!({"type":"rate_limit_event","session_id":"owned","rate_limit_info":{"status":"allowed","rateLimitType":"five_hour","utilization":utilization,"resetsAt":2000}})
}
#[test]
fn stream_utilization_is_fraction_and_missing_is_unknown() {
    let value = quota::parse_claude_event(&event(json!(0.27)), 1000).unwrap();
    assert_eq!(value.view(1001).windows[0].remaining_percent, Some(73.0));
    assert_eq!(value.windows[0].window_minutes, Some(300));
    assert_eq!(value.windows[0].resets_at, Some(2000));
    for status in ["allowed", "allowed_warning", "rejected"] {
        let mut data = event(serde_json::Value::Null);
        data["rate_limit_info"]["status"] = status.into();
        assert_eq!(
            quota::parse_claude_event(&data, 1000).unwrap().windows[0].remaining_percent,
            None
        );
    }
    for used in [json!(-0.1), json!(1.1), json!("0.27")] {
        assert!(quota::parse_claude_event(&event(used), 1000).is_err());
    }
    let mut data = event(json!(0.27));
    data["rate_limit_info"]["rateLimitType"] = "invented".into();
    assert!(quota::parse_claude_event(&data, 1000).is_err());
}
#[tokio::test]
async fn owned_live_stream_persists_before_completion_and_invalid_metadata_does_not_abort() {
    let root = std::env::temp_dir().join(format!(
        "ac-claude-quota-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let spool = root.join("stdout.jsonl");
    let db = root.join("state.sqlite");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let mut valid = event(json!(0.27));
    valid["rate_limit_info"]["resetsAt"] = json!(now + 1000);
    let stream = format!("{}\n{}\n", event(json!(-1)), valid);
    let released = std::sync::atomic::AtomicBool::new(false);
    let worker = async {
        std::fs::write(&spool, stream).unwrap();
        while !released.load(std::sync::atomic::Ordering::SeqCst) {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        42
    };
    let p = profile();
    let observer = agentconductor::progress::observe_profile(worker, &spool, &db, "task", 1, &p);
    let check = async {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            let store = Store::open(&db).unwrap();
            if quota::stored(&store, &p, now).unwrap()["windows"][0]["remaining_percent"] == 73.0 {
                assert_eq!(store.task_events("task", 1).unwrap()[1]["kind"], "usage");
                released.store(true, std::sync::atomic::Ordering::SeqCst);
                break;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "quota not persisted while worker was active"
            );
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    };
    let (out, ()) = tokio::join!(observer, check);
    assert_eq!(out.unwrap(), 42);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn new_event_replaces_windows_and_profile_changes_invalidate() {
    let root = std::env::temp_dir().join(format!(
        "ac-claude-store-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let mut store = Store::open(&root).unwrap();
    let p = profile();
    quota::persist(
        &mut store,
        &p,
        quota::parse_claude_event(&event(json!(0.27)), 1000),
    )
    .unwrap();
    quota::persist(
        &mut store,
        &p,
        quota::parse_claude_event(&event(serde_json::Value::Null), 1001),
    )
    .unwrap();
    assert!(quota::stored(&store, &p, 1002).unwrap()["windows"][0]["remaining_percent"].is_null());
    let mut changed = p.clone();
    changed.executable = "different-login".into();
    assert_eq!(
        quota::stored(&store, &changed, 1002).unwrap()["status"],
        "unavailable"
    );
    drop(store);
    std::fs::remove_file(root).unwrap();
}
#[cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
#[cfg(unix)]
#[tokio::test]
async fn refresh_keeps_original_timestamp_and_revision_without_launching_cli() {
    let (root, mut plan) = support::setup("shared");
    plan.agents.push(profile());
    let run = root.join("run");
    let report = agentconductor::workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    let before = agentconductor::workflow::revision(&report).unwrap();
    let mut store = Store::open(run.join("state.sqlite")).unwrap();
    quota::persist(
        &mut store,
        &profile(),
        quota::parse_claude_event(&event(json!(0.27)), 1000),
    )
    .unwrap();
    drop(store);
    agentconductor::workflow::refresh_quota(&run, "claude")
        .await
        .unwrap();
    let store = Store::open(run.join("state.sqlite")).unwrap();
    let data = quota::stored(&store, &profile(), 1301).unwrap();
    assert_eq!(data["observed_at"], 1000);
    assert_eq!(data["status"], "stale");
    assert_eq!(
        before,
        agentconductor::workflow::revision(&agentconductor::workflow::load(&run).unwrap()).unwrap()
    );
    let room = agentconductor::ui::Room::open(
        &run,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let board = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(board["quotas"]["claude"]["observed_at"], 1000);
    assert_eq!(board["quotas"]["claude"]["status"], "stale");
    drop(room);
    assert!(!std::fs::read_dir(&run).unwrap().any(|p| {
        p.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("quota-query-")
    }));
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn claude_decoder_classifies_quota_without_treating_it_as_task_completion() {
    let mut decoder = agentconductor::protocol::Decoder::new(Provider::Claude, 1024);
    let events = decoder
        .push(format!("{}\n", event(json!(0.27))).as_bytes())
        .unwrap();
    assert_eq!(events[0].kind, agentconductor::protocol::EventKind::Usage);
}

#[cfg(unix)]
#[tokio::test]
async fn subprocess_workflow_delivers_claude_quota_to_native_board() {
    let (root, mut plan) = support::setup("worktree");
    let executable = root.join("claude-quota-fixture");
    assert!(
        std::process::Command::new("rustc")
            .args(["tests/support/claude_quota_agent.rs", "-o"])
            .arg(&executable)
            .status()
            .unwrap()
            .success()
    );
    plan.agents[0].parser = Provider::Claude;
    plan.agents[0].executable = executable.to_str().unwrap().into();
    plan.agents[0].args.clear();
    let run = root.join("run");
    let report = agentconductor::workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert!(report.tasks.iter().all(|t| t.status == "completed"));
    let room = agentconductor::ui::Room::open(
        &run,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let board = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(
        board["quotas"]["mock"]["windows"][0]["remaining_percent"],
        73.0
    );
    assert_eq!(
        board["quotas"]["mock"]["windows"][0]["label"],
        "seven_day_sonnet"
    );
    assert_eq!(
        board["quotas"]["mock"]["source"],
        "claude stream-json rate_limit_event"
    );
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
