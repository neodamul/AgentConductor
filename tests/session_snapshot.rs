//! #955: Workspace snapshots expose only a cheap session-relevant revision.
#[allow(dead_code)]
#[path = "support/goal_actions.rs"]
mod support;
use serde_json::{Value, json};
fn store(run: &std::path::Path, report: &Value) {
    std::fs::write(run.join("report.json"), serde_json::to_vec(report).unwrap()).unwrap();
    agentconductor::state::Store::open(run.join("state.sqlite"))
        .unwrap()
        .record("workflow", report, "session fixture")
        .unwrap();
}
#[test]
fn session_revision_changes_for_started_tasks_without_changing_run_state() {
    let mut f = support::Fixture::new();
    let run = f.report("waiting_approval");
    let mut report: Value =
        serde_json::from_slice(&std::fs::read(run.join("report.json")).unwrap()).unwrap();
    report["plan"]["tasks"] = json!([{"id":"t1","agent":"codex","prompt":"Fixture","paths":[],"depends_on":[],"test_command":[]}]);
    report["tasks"] = json!([{"id":"t1","status":"running","cwd":f.root.join("project"),"started_ms":null,"finished_ms":null,"session_id":null,"error":null,"base":"base","snapshot":null,"artifacts":run.join("t1")}]);
    store(&run, &report);
    f.edit_saved(|s| s["last_run"] = json!(run));
    let before = f.snapshot();
    assert!(before["session_revision"].is_string());
    assert_eq!(f.snapshot()["session_revision"], before["session_revision"]);
    report["tasks"][0]["started_ms"] = json!(100);
    store(&run, &report);
    let started = f.snapshot();
    assert_eq!(started["run_state"], before["run_state"]);
    assert_ne!(started["session_revision"], before["session_revision"]);
    report["error"] = json!("unrelated diagnostic");
    store(&run, &report);
    assert_eq!(
        f.snapshot()["session_revision"],
        started["session_revision"]
    );
    report["tasks"][0]["status"] = json!("completed");
    store(&run, &report);
    assert_ne!(
        f.snapshot()["session_revision"],
        started["session_revision"]
    );
}
