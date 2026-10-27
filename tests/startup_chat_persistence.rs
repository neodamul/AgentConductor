use agentconductor::ui::setup::Startup;
use serde_json::{Value, json};
use std::{fs, path::Path};

fn fixture() -> (std::path::PathBuf, Startup) {
    let root = std::env::temp_dir().join(format!(
        "ac-chat-persist-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let startup = Startup::open(&root, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    (root, startup)
}

fn snapshot(startup: &Startup) -> Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}

fn select_repository(startup: &Startup, repository: &Path) {
    let state = snapshot(startup);
    let mut config = state["config"].clone();
    config["repository"] = repository.to_string_lossy().into_owned().into();
    startup
        .dispatch(
            &json!({"kind":"save","revision":state["revision"],"config":config})
                .to_string()
                .into_bytes(),
        )
        .unwrap();
}

#[test]
fn transcript_is_restored_for_its_repository_without_leaking_to_another() {
    let (root, startup) = fixture();
    let repo_a = root.join("repo-a");
    let repo_b = root.join("repo-b");
    fs::create_dir(&repo_a).unwrap();
    fs::create_dir(&repo_b).unwrap();
    select_repository(&startup, &repo_a);
    drop(startup);

    let settings = root.join("startup.json");
    let mut stored: Value = serde_json::from_slice(&fs::read(&settings).unwrap()).unwrap();
    stored["chat_transcript"] = json!([{"role":"operator","text":"Repo A question"}]);
    fs::write(&settings, serde_json::to_vec(&stored).unwrap()).unwrap();
    let startup = Startup::open(&root, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();

    select_repository(&startup, &repo_b);
    assert!(
        snapshot(&startup)["chat_transcript"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    select_repository(&startup, &repo_a);
    let restored = snapshot(&startup);
    let turns = restored["chat_transcript"].as_array().unwrap();
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0]["text"], "Repo A question");
    drop(startup);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_completed_answer_is_persisted_once_across_repeated_snapshots() {
    let (root, startup) = fixture();
    drop(startup);
    let settings = root.join("startup.json");
    let mut stored: Value = serde_json::from_slice(&fs::read(&settings).unwrap()).unwrap();
    let id = "fixture-chat";
    stored["chat_transcript"] = json!([{"role":"operator","text":"Question"}]);
    stored["chat_result"] = json!(format!("chat-{id}.json"));
    stored["job"] = json!({"id":id,"operation":"chat","started_ms":1});
    fs::write(&settings, serde_json::to_vec(&stored).unwrap()).unwrap();
    fs::create_dir_all(root.join("mcp")).unwrap();
    fs::write(
        root.join("mcp/chat-fixture-chat.json"),
        br#"{"answer":"Answer"}"#,
    )
    .unwrap();
    let process = root.join(format!("coordinator-ui-{id}/process"));
    fs::create_dir_all(&process).unwrap();
    fs::write(
        process.join("owner.json"),
        serde_json::to_vec(&json!({"coordinator_pid":i32::MAX,"pid":null,"process_group":null,"phase":"terminal","exit_code":0,"stopped":false,"timed_out":false})).unwrap(),
    )
    .unwrap();

    let startup = Startup::open(&root, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    let packet = br#"{"kind":"snapshot","chat_result":true}"#;
    let first = startup.dispatch(packet).unwrap();
    assert_eq!(first["chat"]["answer"], "Answer");
    assert_eq!(first["chat_transcript"].as_array().unwrap().len(), 2);
    let second = startup.dispatch(packet).unwrap();
    let turns = second["chat_transcript"].as_array().unwrap();
    assert_eq!(
        turns
            .iter()
            .filter(|turn| turn["role"] == "assistant")
            .count(),
        1
    );
    assert_eq!(turns[1]["text"], "Answer");
    drop(startup);
    fs::remove_dir_all(root).unwrap();
}
