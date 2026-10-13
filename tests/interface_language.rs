#![cfg(unix)]
//! R-70 / DD-76 / QAS-63: English by default, Korean on demand, UTF-8 throughout. HOME is
//! set once in this test binary so the preference stays inside the fixture.
use agentconductor::ui::Room;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const HTML: &str = include_str!("../src/ui/startup.html");
const ROOM: &str = include_str!("../src/ui/room.html");

fn call(room: &Room, value: serde_json::Value) -> Result<serde_json::Value, String> {
    room.dispatch(value.to_string().as_bytes())
}

#[test]
fn settings_menu_offers_english_and_korean() {
    let titlebar = &HTML[..HTML.find("</header>").unwrap()];
    assert!(titlebar.contains("id=\"start-settings-button\""));
    let dialog = &HTML[HTML
        .find("<dialog id=\"start-settings\"")
        .expect("settings dialog")..];
    let dialog = &dialog[..dialog.find("</dialog>").unwrap()];
    assert!(dialog.contains("<option value=\"en\">English</option>"));
    assert!(dialog.contains("<option value=\"ko\">한국어</option>"));
    assert!(ROOM.contains("<meta charset=\"utf-8\">"));
}

#[test]
fn language_is_saved_embedded_and_validated() {
    let (root, _plan) = support::setup("shared");
    unsafe { std::env::set_var("HOME", root.join("home")) };
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = Room::startup(&root.join("desktop"), exe).unwrap();
    let page = room.document().unwrap();
    assert!(page.contains("<html lang=\"en\">"), "English by default");
    assert!(
        page.contains("const AC_MESSAGES={en:") && page.contains("function acMessage("),
        "translation layer ships in the page"
    );
    assert_eq!(
        call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["language"],
        "en"
    );

    assert!(
        call(
            &room,
            serde_json::json!({"kind":"set_language","language":"xx"})
        )
        .is_err()
    );
    call(
        &room,
        serde_json::json!({"kind":"set_language","language":"ko"}),
    )
    .unwrap();
    let stored =
        std::fs::read_to_string(root.join("home/.agentconductor/preferences.json")).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stored).unwrap()["language"],
        "ko"
    );
    drop(room);
    // A new window starts in the saved language.
    let room = Room::startup(&root.join("desktop"), exe).unwrap();
    assert!(room.document().unwrap().contains("<html lang=\"ko\">"));
    assert_eq!(
        call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["language"],
        "ko"
    );
    call(
        &room,
        serde_json::json!({"kind":"set_language","language":"en"}),
    )
    .unwrap();
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
