//! R-45 / DD-53 / QAS-38: planning shows coordinator, elapsed time, ready and failure states.
use agentconductor::ui::setup::Startup;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const HTML: &str = include_str!("../src/ui/startup.html");
const SCRIPT: &str = include_str!("../src/ui/startup.js");
const EN_MESSAGES: &str = include_str!("../src/ui/messages/en.json");
const KO_MESSAGES: &str = include_str!("../src/ui/messages/ko.json");

fn snapshot(startup: &Startup) -> serde_json::Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(startup: &Startup, mut packet: serde_json::Value) -> Result<serde_json::Value, String> {
    packet["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(packet.to_string().as_bytes())
}

#[test]
fn review_area_renders_progress_ready_and_failure() {
    let review = &HTML[HTML.find("id=\"start-panel-review\"").unwrap()..];
    assert!(review.contains("id=\"start-plan-progress\""));
    for text in [
        "\"startup.planning_with_elapsed\"",
        "started_ms",
        "scrollIntoView",
        "\"startup.plan_ready\"",
        "job.log",
    ] {
        assert!(SCRIPT.contains(text), "missing {text}");
    }
}

#[test]
fn planning_preparation_polls_use_the_plan_timeout() {
    let poll = SCRIPT
        .split("async function poll(")
        .nth(1)
        .unwrap()
        .lines()
        .next()
        .unwrap();
    assert!(
        poll.contains("mutating||planStarting?PLAN_START_LIMIT:undefined"),
        "background refresh must not use the short interactive timeout while plan startup is active"
    );
}

#[test]
fn planning_preparation_keeps_one_timeout_across_save_and_baseline_checks() {
    assert!(SCRIPT.contains("async function save(quiet,limit)"));
    assert!(SCRIPT.contains("if(!await save(true,PLAN_START_LIMIT))return"));
    assert!(SCRIPT.contains("async function ensureBaseline(limit=PLAN_START_LIMIT)"));
    assert!(SCRIPT.contains("request({kind:'baseline_status'},limit)"));
}

#[test]
fn timeout_notice_points_to_the_existing_automatic_recovery() {
    for messages in [EN_MESSAGES, KO_MESSAGES] {
        let messages: serde_json::Value = serde_json::from_str(messages).unwrap();
        let text = messages["startup.native_response_timed_out_refresh_before_retrying"]
            .as_str()
            .unwrap();
        assert!(
            text.contains("{value1}"),
            "the timed-out request kind is shown (#200)"
        );
        assert!(text.contains("automatically") || text.contains("자동 확인"));
        assert!(text.contains("reopen") || text.contains("다시 열어"));
        assert!(!text.contains("refresh before retrying") && !text.contains("새로고침하세요"));
    }
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
