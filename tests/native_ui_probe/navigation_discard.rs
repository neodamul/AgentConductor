use super::Probe;
use serde_json::json;

#[test]
fn discard_restores_the_saved_workspace_and_drops_only_local_drafts() {
    let mode = serde_json::to_value(crate::plan::WorkspaceMode::default()).unwrap();
    let roles = serde_json::to_value(crate::roles::RoleRules::default()).unwrap();
    let snapshot = json!({
        "config": {
            "name": "Saved", "repository": "/saved", "goal": "Saved goal",
            "workspace_mode": mode, "coordinator": "saved-agent",
            "profiles": [{"id":"saved-agent"}], "roles": roles
        },
        "proposal": {
            "digest": "saved-digest",
            "plan": {"tasks": [{"id":"saved-task"}], "workspace_mode": mode}
        }
    });
    let mut probe = Probe::default();
    probe.snapshot = Some(snapshot);
    probe.name = "Draft".into();
    probe.repository = "/draft".into();
    probe.goal = "Draft goal".into();
    probe.profiles = vec![json!({"id":"draft-agent"})];
    probe.task_draft = crate::native_ui_probe::workspace::TaskDraft::from_snapshot(
        probe.snapshot.as_ref().unwrap(),
    );
    probe.navigation.name = "New workspace draft".into();
    probe.workspace_step = 2;

    probe.discard_workspace_changes();

    assert_eq!(probe.name, "Saved");
    assert_eq!(probe.repository, "/saved");
    assert_eq!(probe.goal, "Saved goal");
    assert_eq!(probe.profiles, vec![json!({"id":"saved-agent"})]);
    assert!(probe.task_draft.is_none());
    assert!(probe.navigation.name.is_empty());
    assert_eq!(probe.workspace_step, 0);
    assert!(probe.worker.is_none());
    assert!(probe.pending.is_none());
}
