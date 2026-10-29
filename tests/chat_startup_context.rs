#![cfg(unix)]

use agentconductor::ui::setup::Startup;
use serde_json::{Value, json};
use std::{os::unix::fs::PermissionsExt, path::Path, time::Duration};

fn snapshot(startup: &Startup) -> Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}

fn dispatch(startup: &Startup, mut request: Value) -> Value {
    request["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(request.to_string().as_bytes()).unwrap()
}

#[test]
fn workspace_goal_and_project_folder_reach_the_read_only_chat_cli() {
    let id = agentconductor::native_guard::session_id().unwrap();
    let root = std::env::temp_dir().join(format!("ac-chat-context-{id}"));
    let project = root.join("project");
    std::fs::create_dir_all(&project).unwrap();
    let script = root.join("coordinator.sh");
    std::fs::write(
        &script,
        r##"#!/bin/sh
cat > "$(dirname "$0")/prompt.seen"
echo '{"type":"result","is_error":false,"result":"{\"answer\":\"Run cargo test\"}"}'
"##,
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let desktop = root.join("desktop");
    let startup = Startup::open(&desktop, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    let mut config = snapshot(&startup)["config"].clone();
    config["repository"] = json!(project);
    config["goal"] = json!("Build the ladder game");
    config["profiles"] = json!([{
        "id":"coordinator",
        "executable":script,
        "args":[],
        "parser":"claude",
        "permission_mode":"read_only"
    }]);
    config["coordinator"] = json!("coordinator");
    dispatch(&startup, json!({"kind":"save","config":config}));
    dispatch(
        &startup,
        json!({
            "kind":"chat",
            "text":"What was built?"
        }),
    );

    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    loop {
        let result = startup
            .dispatch(br#"{"kind":"snapshot","chat_result":true}"#)
            .unwrap();
        if result["chat"]["answer"] == "Run cargo test" {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "{result}");
        std::thread::sleep(Duration::from_millis(50));
    }
    let prompt = std::fs::read_to_string(root.join("prompt.seen")).unwrap();
    assert!(prompt.contains("## Workspace"), "{prompt}");
    assert!(prompt.contains("Build the ladder game"), "{prompt}");
    assert!(prompt.contains(project.to_str().unwrap()), "{prompt}");
    assert!(prompt.contains("No run has started yet"), "{prompt}");

    let history = startup
        .dispatch(br#"{"kind":"snapshot","chat_history":true,"chat_result":true}"#)
        .unwrap();
    assert_eq!(
        history["chat_history"],
        json!([
            {"role":"operator","text":"What was built?"},
            {"role":"assistant","text":"Run cargo test"}
        ])
    );
    drop(startup);

    let startup = Startup::open(&desktop, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    let restored = startup
        .dispatch(br#"{"kind":"snapshot","chat_history":true}"#)
        .unwrap();
    assert_eq!(restored["chat_history"], history["chat_history"]);
    startup
        .dispatch(
            json!({
                "kind":"chat",
                "revision":restored["chat_revision"],
                "text":"What should I do next?"
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap();

    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    loop {
        let result = startup
            .dispatch(br#"{"kind":"snapshot","chat_result":true}"#)
            .unwrap();
        if result["chat"]["answer"] == "Run cargo test" {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "{result}");
        std::thread::sleep(Duration::from_millis(50));
    }
    let prompt = std::fs::read_to_string(root.join("prompt.seen")).unwrap();
    assert!(prompt.contains("What was built?"), "{prompt}");
    assert!(prompt.contains("Run cargo test"), "{prompt}");
    assert!(prompt.contains("What should I do next?"), "{prompt}");
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
