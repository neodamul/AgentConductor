#![cfg(unix)]
//! GitHub #261 (DD-204): the chat runs in its own job slot, so it works while a plan is being made.
use agentconductor::ui::Room;
use serde_json::{Value, json};
use std::{os::unix::process::CommandExt, path::Path, process::Command};
#[path = "support/env.rs"]
mod env;

fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[test]
fn a_chat_is_accepted_and_followed_while_a_plan_is_running() {
    let root = std::env::temp_dir().join(format!("ac-chat-slot-{}", std::process::id()));
    let (repo, desktop) = (root.join("repo"), root.join("desktop"));
    std::fs::create_dir_all(&repo).unwrap();
    let coordinator = root.join("coordinator.sh");
    std::fs::write(&coordinator, "#!/bin/sh\nexec sleep 5\n").unwrap();
    std::fs::set_permissions(
        &coordinator,
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    )
    .unwrap();
    let mut environment = env::lock();
    environment.set("HOME", root.join("home"));
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    drop(Room::startup(&desktop, exe).unwrap());

    // A planning job that is still alive: a stand-in process owned like the planner's.
    let planner = Command::new("sleep").arg("30").process_group(0).spawn();
    let mut planner = planner.unwrap();
    let id = "plan-job-1";
    let process = desktop.join(format!("coordinator-ui-{id}/process"));
    std::fs::create_dir_all(&process).unwrap();
    std::fs::write(
        process.join("owner.json"),
        json!({"coordinator_pid":std::process::id(),"pid":planner.id(),"process_group":planner.id(),
            "phase":"running","exit_code":null,"stopped":false,"timed_out":false})
        .to_string(),
    )
    .unwrap();
    let path = desktop.join("startup.json");
    let mut saved = read(&path);
    saved["config"]["profiles"] = json!([{"id":"coordinator","executable":coordinator,"args":[],"parser":"claude","permission_mode":"read_only"}]);
    saved["config"]["coordinator"] = "coordinator".into();
    saved["config"]["repository"] = json!(repo);
    saved["config"]["goal"] = "answer questions".into();
    saved["job"] = json!({"id":id,"operation":"plan","started_ms":1});
    std::fs::write(&path, saved.to_string()).unwrap();

    let room = Room::startup(&desktop, exe).unwrap();
    let before = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!((before["job"]["kind"].as_str(), before["job"]["state"].as_str()), (Some("plan"), Some("running")));
    assert!(before["chat_job"].is_null(), "no chat has run");
    let ask = |revision: &Value| {
        room.dispatch(json!({"kind":"chat","revision":revision,"text":"Why is the plan slow?"}).to_string().as_bytes())
    };
    let accepted = ask(&before["chat_revision"]).expect("a chat is not refused because a plan is running");
    assert_eq!(accepted["kind"], "chat");

    let during = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(during["job"]["id"], id, "the plan job is untouched");
    assert_eq!(during["job"]["state"], "running");
    assert_eq!(during["chat_job"]["id"], accepted["id"], "the chat is followed in its own slot");
    assert_eq!(during["chat_job"]["state"], "running");
    assert!(ask(&during["chat_revision"]).is_err(), "one chat at a time");
    let reset = room.dispatch(br#"{"kind":"chat_reset"}"#);
    assert!(reset.is_err(), "the conversation is not cleared under a running chat");

    drop(room);
    planner.kill().unwrap();
    planner.wait().unwrap();
    let _ = std::fs::remove_dir_all(root);
}
