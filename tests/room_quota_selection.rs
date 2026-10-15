//! User requests 2026-10-14: the Agent Room's account usage shows Enterprise budgets as a
//! remaining percentage like the agent cards, and the task shown on the right has a strong
//! border in the list.
const ROOM: &str = include_str!("../src/ui/room.js");
const CSS: &str = include_str!("../src/ui/room.css");

#[test]
fn enterprise_budgets_are_rendered_from_rust_snapshot_rows() {
    assert!(ROOM.contains("snapshot.quota_budget_rows?.[agent?.id]"));
    assert!(!ROOM.contains("const share=b=>"));
}

#[test]
fn the_selected_task_has_a_strong_border() {
    assert!(CSS.contains(".session.active{outline:2px solid var(--accent);outline-offset:-2px}"));
}
