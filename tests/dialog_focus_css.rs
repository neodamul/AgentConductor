//! #546: a dialog opened by a click shows no focus ring on its automatically focused control until a key is pressed.
const SCRIPT: &str = include_str!("../src/ui/dialog_focus.js");
const ROOM: &str = include_str!("../src/ui/room.css");

#[test]
fn a_dialog_opened_by_a_click_shows_no_focus_ring_until_a_key_is_pressed() {
    assert!(SCRIPT.contains("HTMLDialogElement.prototype.showModal"));
    assert!(SCRIPT.contains("classList.add('quiet-focus')") && SCRIPT.contains("'keydown'"));
    assert!(
        !SCRIPT.contains(".focus("),
        "the focus is never moved: confirmation dialogs focus Cancel on purpose"
    );
    assert!(ROOM.contains("dialog.quiet-focus :focus-visible{outline:none}"));
    assert!(include_str!("../src/ui/setup/document.rs").contains("dialog_focus.js"));
    assert!(include_str!("../src/ui/document.rs").contains("dialog_focus.js"));
}
