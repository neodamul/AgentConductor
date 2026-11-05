//! #546: a dialog opened by a click shows no focus ring on its first button or check box.
const SCRIPT: &str = include_str!("../src/ui/dialog_focus.js");
const ROOM: &str = include_str!("../src/ui/startup.css");

#[test]
fn a_dialog_opened_by_a_click_shows_no_focus_ring_on_its_first_control() {
    assert!(SCRIPT.contains("HTMLDialogElement.prototype.showModal"));
    assert!(
        SCRIPT.contains("input[type=checkbox]")
            && SCRIPT.contains("this.focus({ preventScroll: true })")
    );
    assert!(ROOM.contains("dialog:focus{outline:none}"));
    assert!(include_str!("../src/ui/setup/document.rs").contains("dialog_focus.js"));
}
