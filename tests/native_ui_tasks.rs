use agentconductor::{
    native_ui_probe::workspace::{TaskDraft, task_controls, task_edit_request},
    plan::WorkspaceMode,
};
use eframe::egui;

fn snapshot() -> serde_json::Value {
    serde_json::json!({
        "revision":"workspace-rev-9",
        "proposal":{
            "phase":"ready",
            "digest":"plan-digest-9",
            "plan":{
                "goal":"Implement token rotation",
                "agents":[{"id":"codex","executable":"codex","args":[]}]
            }
        }
    })
}

fn task() -> serde_json::Value {
    serde_json::json!({
        "id":"auth-tests",
        "agent":"codex",
        "model":null,
        "prompt":"Add rotation tests\nDone when: replay is rejected",
        "paths":["tests/auth.rs"],
        "depends_on":[],
        "test_command":["cargo","test"]
    })
}

#[test]
fn plan_edit_request_binds_revision_digest_tasks_and_mode() {
    let packet = task_edit_request(
        &snapshot(),
        &[task()],
        WorkspaceMode::Worktree,
        "plan-digest-9",
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["kind"], "edit");
    assert_eq!(value["revision"], "workspace-rev-9");
    assert_eq!(value["tasks"][0]["id"], "auth-tests");
    assert_eq!(value["tasks"][0]["paths"][0], "tests/auth.rs");
    assert_eq!(value["workspace_mode"], "worktree");
}

#[test]
fn plan_edit_request_rejects_stale_or_invalid_task_edits() {
    assert!(
        task_edit_request(&snapshot(), &[task()], WorkspaceMode::Shared, "old-digest").is_err()
    );
    assert!(task_edit_request(&snapshot(), &[], WorkspaceMode::Shared, "plan-digest-9").is_err());
    let mut invalid = task();
    invalid["agent"] = "unregistered".into();
    assert!(
        task_edit_request(
            &snapshot(),
            &[invalid],
            WorkspaceMode::Shared,
            "plan-digest-9"
        )
        .is_err()
    );
    let mut invalid = task();
    invalid["test_command"] = serde_json::json!([]);
    assert!(
        task_edit_request(
            &snapshot(),
            &[invalid],
            WorkspaceMode::Shared,
            "plan-digest-9"
        )
        .is_err()
    );
}

#[test]
fn task_editor_exposes_assignment_instructions_scope_dependencies_and_command() {
    let mut value = snapshot();
    value["proposal"]["plan"]["agents"] = serde_json::json!([{
        "id":"codex", "executable":"codex", "args":[]
    }]);
    value["proposal"]["plan"]["tasks"] = serde_json::json!([task()]);
    value["proposal"]["plan"]["workspace_mode"] = serde_json::json!("shared");
    let mut draft = TaskDraft::from_snapshot(&value);
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        let _ = task_controls(&value, &mut draft, false, ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for label in [
        "Assigned agent",
        "Session model",
        "Instructions",
        "Owned paths (one per line)",
        "Dependencies (comma separated)",
        "Test command (JSON argv)",
    ] {
        assert!(labels.iter().any(|actual| actual == label), "{label}");
    }
}

#[test]
fn dispatcher_persists_edited_plan_and_refuses_a_stale_digest() {
    let (root, plan) = support::setup("shared");
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = agentconductor::ui::Room::startup(&root.join("desktop"), exe).unwrap();
    let before = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let mut profiles = plan.agents;
    let mut planner = profiles[0].clone();
    planner.id = "planner".into();
    planner.args = vec!["plan".into()];
    planner.permission_mode = "read_only".into();
    profiles.push(planner);
    let save = serde_json::json!({
        "kind":"save", "revision":before["revision"], "config":{
            "profiles":profiles, "repository":root.join("repo"),
            "goal":"Implement scoped tasks", "coordinator":"planner", "workspace_mode":"shared"
        }
    });
    room.dispatch(&serde_json::to_vec(&save).unwrap()).unwrap();
    let configured = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    room.dispatch(
        &serde_json::to_vec(&serde_json::json!({
            "kind":"prepare", "revision":configured["revision"]
        }))
        .unwrap(),
    )
    .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let ready = loop {
        let current = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
        if current["job"]["state"] != "running" {
            assert_eq!(current["proposal"]["phase"], "ready", "{current}");
            break current;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "plan generation timed out"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    let mut tasks = ready["proposal"]["plan"]["tasks"]
        .as_array()
        .unwrap()
        .clone();
    tasks[0]["prompt"] = "Updated reviewed instructions".into();
    let digest = ready["proposal"]["digest"].as_str().unwrap();
    let packet = task_edit_request(&ready, &tasks, WorkspaceMode::Shared, digest).unwrap();
    let mut stale: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    stale["digest"] = "stale".into();
    assert!(room.dispatch(&serde_json::to_vec(&stale).unwrap()).is_err());
    assert_eq!(room.dispatch(&packet).unwrap()["saved"], true);
    let updated = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(
        updated["proposal"]["plan"]["tasks"][0]["prompt"],
        "Updated reviewed instructions"
    );
    assert_ne!(updated["proposal"]["digest"], digest);
    drop(room);
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::remove_dir_all(root).unwrap();
}

#[path = "support/workflow.rs"]
mod support;
