use serde_json::Value;

pub fn prepare_request(revision: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"kind":"prepare","revision":revision}))
        .expect("prepare request is serializable")
}

pub fn task_edit_request(
    snapshot: &Value,
    tasks: &[Value],
    workspace_mode: crate::plan::WorkspaceMode,
    digest: &str,
) -> Result<Vec<u8>, String> {
    super::tasks::edit_request(snapshot, tasks, workspace_mode, digest)
}

pub fn workspace_save_request(
    snapshot: &Value,
    name: &str,
    repository: &str,
    goal: &str,
    workspace_mode: crate::plan::WorkspaceMode,
    profiles: &[Value],
    coordinator: &str,
) -> Vec<u8> {
    let mut config = snapshot["config"].clone();
    config["name"] = name.into();
    config["repository"] = repository.into();
    config["goal"] = goal.into();
    config["workspace_mode"] = serde_json::to_value(workspace_mode).unwrap();
    config["profiles"] = serde_json::to_value(profiles).unwrap();
    config["coordinator"] = coordinator.into();
    serde_json::to_vec(&serde_json::json!({
        "kind": "save",
        "revision": snapshot["revision"],
        "config": config
    }))
    .expect("workspace save request is serializable")
}

pub fn picked_folder(current: &str, response: &Value) -> String {
    response["picked_folder"]
        .as_str()
        .unwrap_or(current)
        .to_owned()
}
