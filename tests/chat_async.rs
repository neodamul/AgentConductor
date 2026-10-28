#![cfg(unix)]
//! GitHub #197: a slow coordinator never blocks the screen. Sending returns at once, a second send is refused
//! while the first runs, the answer arrives in a later snapshot and the next question carries the conversation.
use agentconductor::ui::setup::Startup;
use serde_json::{Value, json};
use std::{os::unix::fs::PermissionsExt, path::Path, time::Duration};
#[path = "support/workflow.rs"]
mod support;

fn snapshot(startup: &Startup) -> Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(startup: &Startup, mut packet: Value) -> Result<Value, String> {
    packet["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(packet.to_string().as_bytes())
}
fn wait_for_answer(startup: &Startup) -> Value {
    for _ in 0..300 {
        let state = snapshot(startup);
        if !state["chat"].is_null() {
            return state;
        }
        assert_ne!(state["job"]["state"], "failed", "{state}");
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("no answer within 30 s");
}

#[test]
fn a_slow_coordinator_answers_later_and_the_conversation_continues() {
    let (root, _) = support::setup("shared");
    let script = root.join("coordinator.sh");
    // Each call keeps its prompt, thinks for two seconds and answers with its own number.
    std::fs::write(
        &script,
        "#!/bin/sh\nd=\"$(dirname \"$0\")\"\ncat > \"$d/prompt.$$\"\nsleep 2\nn=$(ls \"$d\" | grep -c '^prompt\\.')\n\
         echo \"{\\\"type\\\":\\\"result\\\",\\\"is_error\\\":false,\\\"result\\\":\\\"{\\\\\\\"answer\\\\\\\":\\\\\\\"Answer-$n\\\\\\\"}\\\"}\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let profile = json!({"id":"coordinator","executable":script,"args":[],"parser":"claude","permission_mode":"read_only"});
    mutate(&startup, json!({"kind":"save","config":{"profiles":[profile],"repository":root.join("repo"),"goal":"g","coordinator":"coordinator","workspace_mode":"shared"}})).unwrap();

    let started = std::time::Instant::now();
    mutate(&startup, json!({"kind":"chat","text":"first question"})).unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "sending returns before the coordinator finishes"
    );
    assert!(
        mutate(&startup, json!({"kind":"chat","text":"too soon"})).is_err(),
        "a second question waits for the first"
    );
    assert_eq!(wait_for_answer(&startup)["chat"]["answer"], "Answer-1");
    mutate(&startup, json!({"kind":"chat","text":"second question"})).unwrap();
    assert_eq!(wait_for_answer(&startup)["chat"]["answer"], "Answer-2");
    let turns = startup.dispatch(br#"{"kind":"chat_history"}"#).unwrap()["turns"].clone();
    let text: Vec<_> = turns
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["text"].as_str().unwrap())
        .collect();
    assert_eq!(
        text,
        ["first question", "Answer-1", "second question", "Answer-2"]
    );
    let second = std::fs::read_dir(&root)
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter(|p| p.contains("second question"))
        .count();
    assert!(second >= 1, "the second prompt reached the coordinator");
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
