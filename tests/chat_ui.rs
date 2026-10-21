//! GitHub #115: the workspace page embeds the chat panel, its script and localized text.
use agentconductor::ui::Room;

#[test]
fn the_startup_document_embeds_the_chat_panel_and_script() {
    let root = std::env::temp_dir().join(format!("ac-chat-ui-{}", std::process::id()));
    let room = Room::startup(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let document = room.document().unwrap();
    for needle in [
        "id=\"start-chat-button\"",
        "id=\"start-chat\"",
        "id=\"start-chat-form\"",
        "function chatUi(",
        "chatUi({q,request",
        ".chat-panel{",
    ] {
        assert!(document.contains(needle), "{needle}");
    }
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_chat_panel_script_sends_shows_and_reports() {
    let result = std::process::Command::new("node")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("tests/chat_runtime.cjs")
        .output()
        .expect("Node runs UI behavior tests");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
