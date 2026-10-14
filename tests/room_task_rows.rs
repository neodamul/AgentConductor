//! Compact task rows in the Agent Room. User requests 2026-10-14 (R-128 / DD-137 amendment): the
//! task name leads; a small icon, the agent name and a compact status tag share the next line.
const ROOM: &str = include_str!("../src/ui/room.js");
const CSS: &str = include_str!("../src/ui/room.css");

#[test]
fn the_task_name_leads_and_icon_name_and_status_share_one_line() {
    assert!(ROOM.contains("meta.append(avatar(agent?.parser,agentName(agent),'session-symbol'),node('span',agentName(agent),'name'),node('span',statusLabel(session.status),'sub session-status"));
    // The first line leads with the reviewed task summary and keeps the task ID as a label.
    assert!(ROOM.contains("body.append(taskHeading(session.id,r.plan.tasks[index]?.prompt),meta)"));
    assert!(ROOM.contains("heading.append(node('span',id,'task-id'));if(summary)heading.append(node('span',summary,'task-summary'))"));
    assert!(CSS.contains(".session-choice .session-meta{display:flex;flex-wrap:wrap"));
    assert!(CSS.contains(".session-choice .session-meta .session-symbol,.session-choice .session-meta .app-icon.session-symbol{width:14px;height:14px"));
}
