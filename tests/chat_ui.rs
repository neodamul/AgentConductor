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

#[test]
fn the_page_refreshes_the_chat_when_a_snapshot_is_rendered() {
    let script = include_str!("../src/ui/startup.js");
    assert!(
        script.contains("sources.update();chat.update();AC_I18N.bind(q('status')"),
        "answers only show if the render path updates the chat"
    );
}

#[test]
fn the_chat_button_uses_a_plain_glyph_that_needs_no_emoji_font() {
    // A non-BMP emoji forces font fallback on first paint, which stalled the Windows UI test.
    let html = include_str!("../src/ui/startup.html");
    let button = &html[html.find("id=\"start-chat-button\"").unwrap()..];
    let glyph = &button[button.find('>').unwrap() + 1..button.find("</button>").unwrap()];
    assert!(glyph.chars().all(|c| (c as u32) < 0x2000), "{glyph}");
}
