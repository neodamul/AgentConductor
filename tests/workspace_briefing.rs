//! #427: a cold Workspace briefing reads only its saved run without opening a controller.
#[allow(dead_code)]
#[path = "support/goal_actions.rs"]
mod support;
use serde_json::json;
use support::Fixture;

#[test]
fn workspace_briefing_requires_a_saved_run_and_does_not_advance_state() {
    let mut f = Fixture::new();
    assert!(f.call(json!({"kind":"workspace_briefing"})).is_err());
    let run = f.report("waiting_approval");
    f.edit_saved(|s| s["last_run"] = json!(run));
    let before = f.snapshot()["revision"].clone();
    let report_before = std::fs::read(run.join("report.json")).unwrap();
    let answer = f.call(json!({"kind":"workspace_briefing"})).unwrap();
    assert_eq!(answer["goal"], "First goal");
    assert_eq!(answer["state"], "waiting_approval");
    assert!(answer["sessions"].is_array());
    assert_eq!(f.snapshot()["revision"], before);
    assert_eq!(
        std::fs::read(run.join("report.json")).unwrap(),
        report_before
    );
    assert!(!run.join("availability-coordinator").exists());
    assert!(
        f.call(json!({"kind":"workspace_briefing","revision":"unexpected"}))
            .is_err()
    );
}

#[test]
fn workspace_briefing_rejects_an_outside_run_and_new_goal_clears_its_context() {
    let mut f = Fixture::new();
    let outside = f.root.join("outside-run");
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("report.json"), "{}").unwrap();
    f.edit_saved(|s| s["last_run"] = json!(outside));
    assert!(f.call(json!({"kind":"workspace_briefing"})).is_err());
    assert!(!outside.join("state.sqlite").exists());
    let run = f.report("completed");
    f.edit_saved(|s| s["last_run"] = json!(run));
    assert!(f.call(json!({"kind":"workspace_briefing"})).is_ok());
    f.mutate(json!({"kind":"new_goal"})).unwrap();
    assert!(f.call(json!({"kind":"workspace_briefing"})).is_err());
}

#[test]
fn cold_room_routes_workspace_briefing_without_opening_run_controller() {
    let mut f = Fixture::new();
    let run = f.report("waiting_approval");
    f.edit_saved(|s| s["last_run"] = json!(run));
    drop(f.startup.take());
    let room = agentconductor::ui::Room::startup(
        &f.root.join("workspace"),
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let answer = room.dispatch(br#"{"kind":"workspace_briefing"}"#).unwrap();
    assert_eq!(answer["goal"], "First goal");
    assert!(
        room.dispatch(br#"{"kind":"briefing","room":true}"#)
            .is_err()
    );
    assert!(!run.join("ui-window.lock").exists());
    assert!(!run.join("availability-coordinator").exists());
    drop(room);
}
