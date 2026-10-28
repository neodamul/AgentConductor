#![cfg(unix)]
//! GitHub #200: every `chat` request and every request that takes a second or more is recorded in a bounded log,
//! so a timeout toast can be traced to the request that caused it.
use agentconductor::ui::Room;
use std::path::Path;
#[path = "support/env.rs"]
mod env;

#[test]
fn chat_requests_are_logged_fast_ones_are_not_and_the_log_is_bounded() {
    let root = std::env::temp_dir().join(format!("ac-diagnostics-{}", std::process::id()));
    let home = root.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let mut env = env::lock();
    env.set("HOME", &home);
    let room = Room::startup(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let log = home.join(".agentconductor/request-diagnostics.log");
    room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert!(!log.exists(), "a fast snapshot leaves no trace");
    let revision = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["revision"].clone();
    let packet = serde_json::json!({"kind":"chat","revision":revision,"text":"hello"}).to_string();
    assert!(room.dispatch(packet.as_bytes()).is_err());
    let text = std::fs::read_to_string(&log).unwrap();
    let lines: Vec<_> = text.lines().collect();
    assert_eq!(lines.len(), 2, "{text}");
    assert!(lines[0].contains(" start kind=chat "), "{text}");
    assert!(
        lines[1].contains(" done kind=chat ") && lines[1].contains(" ok=false "),
        "{text}"
    );
    assert!(lines[1].contains("error="), "the reason is kept: {text}");
    std::fs::write(&log, "x".repeat(300 * 1024)).unwrap();
    assert!(room.dispatch(packet.as_bytes()).is_err());
    assert!(
        log.metadata().unwrap().len() < 4096,
        "the log restarts after the limit"
    );
    assert!(
        log.with_extension("log.1").exists(),
        "the previous log is kept once"
    );
    drop(room);
    drop(env);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn both_shipped_manuals_tell_an_agent_how_to_read_the_log() {
    for file in ["docs/user-manual.html", "docs/user-manual.ko.html"] {
        let manual = std::fs::read_to_string(file).unwrap();
        for needle in [
            "request-diagnostics.log",
            "start kind=",
            "done kind=",
            "ok=false",
            "256 KiB",
        ] {
            assert!(manual.contains(needle), "{file} lacks {needle}");
        }
    }
}
