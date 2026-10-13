//! Compact task rows in the Agent Room (user request 2026-10-14): a small icon and the agent
//! name on the left of the first line, the status tag at its right end, the task name below.
const ROOM: &str = include_str!("../src/ui/room.js");
const CSS: &str = include_str!("../src/ui/room.css");

#[test]
fn the_status_tag_shares_the_first_line_with_the_agent_name() {
    assert!(ROOM.contains("heading.append(node('span',agentName(agent),'name'),node('span',statusLabel(session.status),'sub session-status"));
    assert!(ROOM.contains("node('span',session.id,'sub task-name')"));
    assert!(CSS.contains(".session-choice .name-row .session-status{"));
    assert!(CSS.contains(".session-choice .session-symbol,.session-choice .app-icon.session-symbol{width:16px;height:16px"));
}
