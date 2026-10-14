//! User requests 2026-10-14: the Agent Room's account usage shows Enterprise budgets as a
//! remaining percentage like the agent cards, and the task shown on the right has a strong
//! border in the list.
const ROOM: &str = include_str!("../src/ui/room.js");
const CSS: &str = include_str!("../src/ui/room.css");

#[test]
fn budgets_read_as_a_remaining_percentage() {
    assert!(ROOM.contains("const share=b=>typeof b.remaining_percent==='number'"));
    assert!(
        ROOM.contains("percent!==null?acMessage(\"room.remaining\",{value1:Math.round(percent)})")
    );
}

#[test]
fn the_selected_task_has_a_strong_border() {
    assert!(CSS.contains(".session.active{outline:2px solid var(--accent);outline-offset:-2px}"));
}
