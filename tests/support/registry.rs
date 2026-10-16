use agentconductor::ui::Room;
use std::path::Path;

pub fn call(room: &Room, value: serde_json::Value) -> Result<serde_json::Value, String> {
    room.dispatch(value.to_string().as_bytes())
}
pub fn save(
    room: &Room,
    name: &str,
    profiles: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let revision = call(room, serde_json::json!({"kind":"snapshot"}))?["revision"].clone();
    let config = serde_json::json!({"profiles":profiles,"repository":"","goal":"","coordinator":"mock","workspace_mode":"shared","name":name});
    call(
        room,
        serde_json::json!({"kind":"save","revision":revision,"config":config}),
    )
}

pub fn delete_current(room: &Room) -> (std::path::PathBuf, usize) {
    let current = call(room, serde_json::json!({"kind":"workspaces"})).unwrap()["current"]
        .as_str()
        .map(|p| Path::new(p).canonicalize().unwrap())
        .unwrap();
    let listed = call(room, serde_json::json!({"kind":"saved_workspaces"})).unwrap();
    let listed = listed["workspaces"].as_array().unwrap().clone();
    let entry = listed
        .iter()
        .find(|w| {
            Path::new(w["path"].as_str().unwrap())
                .canonicalize()
                .unwrap()
                == current
        })
        .unwrap();
    call(
        room,
        serde_json::json!({"kind":"delete_workspace","path":entry["path"],"name":entry["name"]}),
    )
    .unwrap();
    (current, listed.len())
}
