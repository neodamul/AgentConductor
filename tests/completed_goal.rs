//! #997: completed Goals reject writes before any job or audit mutation.
#[allow(dead_code)]
#[path = "support/goal_actions.rs"]
mod support;
use agentconductor::{ui::Room, workflow};
use serde_json::json;
use std::{fs, path::Path};
fn fixture() -> (support::Fixture, std::path::PathBuf) {
    let mut f = support::Fixture::new();
    let run = f.report("completed");
    f.edit_saved(|s| s["last_run"] = json!(run));
    (f, run)
}
#[test]
fn completed_startup_rejects_goal_writes_without_changing_saved_data() {
    let (f, run) = fixture();
    let snapshot = f.snapshot();
    let saved = fs::read(f.root.join("workspace/startup.json")).unwrap();
    for packet in [
        json!({"kind":"save","config":snapshot["config"]}),
        json!({"kind":"save_auto_merge","enabled":true}),
        json!({"kind":"prepare"}),
        json!({"kind":"draft"}),
        json!({"kind":"commit_baseline"}),
        json!({"kind":"ignore_os_files"}),
        json!({"kind":"read_plan","path":"not-read.json"}),
        json!({"kind":"edit","tasks":[],"workspace_mode":"shared"}),
        json!({"kind":"start","digest":"unused"}),
        json!({"kind":"save_profiles","profiles":[]}),
        json!({"kind":"mcp_apply","preview_id":"unused","selected":[]}),
        json!({"kind":"optimize_goal","goal":"rewrite"}),
        json!({"kind":"import_settings","folder":f.root,"keep_verified":false}),
    ] {
        let error = f.mutate(packet.clone()).unwrap_err();
        assert!(
            error.contains("read-only") || error.contains("읽기 전용"),
            "{packet}: {error}"
        );
        assert_eq!(
            fs::read(f.root.join("workspace/startup.json")).unwrap(),
            saved
        );
        assert_eq!(workflow::load(&run).unwrap().state, "completed");
    }
    assert!(f.call(json!({"kind":"chat_history"})).is_ok());
    assert!(f.call(json!({"kind":"chat_reset"})).is_ok());
    assert!(f.mutate(json!({"kind":"new_goal"})).is_ok());
    assert!(f.snapshot()["run_state"].is_null());
}
#[test]
fn completed_room_refuses_every_mutating_action_before_launch() {
    let (f, run) = fixture();
    let room = Room::open(&run, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    let report = workflow::load(&run).unwrap();
    let revision = workflow::revision(&report).unwrap();
    for action in [
        json!({"kind":"revise","task":"x","feedback":"change"}),
        json!({"kind":"reassign","task":"x","agent":"x"}),
        json!({"kind":"model","task":"x","model":"x"}),
        json!({"kind":"schedule","task":"x","at":"2030-01-01T00:00:00Z"}),
        json!({"kind":"resume"}),
        json!({"kind":"recover"}),
        json!({"kind":"stop","reason":"stop"}),
        json!({"kind":"approve","sha":"x"}),
        json!({"kind":"reject","reason":"no"}),
        json!({"kind":"merge","sha":"x"}),
        json!({"kind":"tool_approve","sha":"x"}),
        json!({"kind":"tool_reject","sha":"x"}),
        json!({"kind":"tool_refresh","sha":"x"}),
        json!({"kind":"coordinator_request","message":"change"}),
        json!({"kind":"scope_recovery","task":"x","paths":[],"digest":"x"}),
    ] {
        let error = room
            .dispatch(
                json!({"kind":"action","room":true,"revision":revision,"action":action})
                    .to_string()
                    .as_bytes(),
            )
            .unwrap_err();
        assert!(
            error.contains("read-only") || error.contains("읽기 전용"),
            "{action}: {error}"
        );
    }
    assert_eq!(workflow::load(&run).unwrap().state, "completed");
    assert!(!run.join("stop.request").exists());
    drop(room);
    drop(f);
}
#[tokio::test]
async fn coordinator_feedback_cannot_queue_or_write_to_a_completed_run() {
    let (_f, run) = fixture();
    let before = fs::read(run.join("report.json")).unwrap();
    let error = workflow::coordinator_request(&run, 1, "change this result")
        .await
        .unwrap_err();
    assert!(error.contains("read-only"), "{error}");
    assert_eq!(fs::read(run.join("report.json")).unwrap(), before);
    assert!(workflow::decisions::list(&run).unwrap().is_empty());
}

#[test]
fn completed_run_cannot_rebind_automatic_merge_consent() {
    let (_f, run) = fixture();
    let report = workflow::load(&run).unwrap();
    let error = workflow::set_auto_merge(&run, true, &report.plan).unwrap_err();
    assert!(error.contains("read-only"));
    assert!(!run.with_extension("auto-merge.json").exists());
}
