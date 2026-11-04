//! Compact task rows in the Agent Room. User requests 2026-10-14 (R-128 / DD-137 amendment): the
//! task name leads; a small icon, the agent name and a compact status tag share the next line.
const ROOM: &str = include_str!("../src/ui/room.js");
const CSS: &str = include_str!("../src/ui/room.css");

#[test]
fn the_task_name_leads_and_icon_name_and_status_share_one_line() {
    assert!(ROOM.contains("meta.append(symbol,node('span',agentName(agent),'name'),node('span',statusLabel(session.status),'sub session-status"));
    // The first line leads with the reviewed task summary and keeps the task ID as a label.
    assert!(
        ROOM.contains("body.append(taskHeading(session.id,r.plan.tasks[index]?.prompt),meta,note)")
    );
    assert!(ROOM.contains("heading.append(node('span',id,'task-id'));if(summary)heading.append(node('span',summary,'task-summary'))"));
    assert!(CSS.contains(".session-choice .session-meta{display:flex;flex-wrap:wrap"));
    assert!(CSS.contains(".session-choice .session-meta .session-symbol,.session-choice .session-meta .app-icon.session-symbol{width:28px;height:28px"));
}

#[test]
fn brand_marks_sit_in_a_bordered_container_with_the_specified_mark_size() {
    // GitHub #406 / visual-spec: task avatar 28px with a 20px mark, agent avatar 29px with a 19px mark,
    // both inside the 1.5px border (28 = 1.5 + 2.5 + 20 + 2.5 + 1.5; 29 = 1.5 + 3.5 + 19 + 3.5 + 1.5).
    assert!(CSS.contains(".session-choice .session-meta .app-icon.session-symbol{padding:2.5px}"));
    assert!(CSS.contains(".app-icon.agent-symbol{width:29px;height:29px;box-sizing:border-box;border:1.5px solid var(--line);padding:3.5px}"));
}

#[test]
fn the_avatar_border_carries_the_task_status() {
    // GitHub #400: the avatar carries the status as a 1.5px border and only a running one pulses.
    assert!(ROOM.contains("symbol.classList.add('st-'+statusLight(session.status))"));
    assert!(ROOM.contains("if(session.status==='running')symbol.classList.add('running')"));
    assert!(!ROOM.contains("agent-light"), "the separate dot is gone");
    assert!(!CSS.contains("agent-light"), "and so are its rules");
    for (name, color) in [("green", "accent"), ("yellow", "warn"), ("red", "err")] {
        let rule = format!(".session-symbol.st-{name}{{border-color:var(--{color})}}");
        assert!(CSS.contains(&rule), "{rule}");
    }
    assert!(CSS.contains("border:1.5px solid"));
    assert!(CSS.contains(".session-symbol.running{animation:agent-pulse"));
    assert!(CSS.contains("prefers-reduced-motion:reduce){.session-choice .session-meta .session-symbol.running{animation:none}"));
}

#[test]
fn approval_and_merge_are_separate_visible_stages_and_merge_keeps_progress_in_the_banner() {
    // GitHub #390: the stages are steps 4-6 of the one stepper (startup.html), marked from the Room.
    let html = include_str!("../src/ui/startup.html");
    for step in ["start-tab-tasks", "start-tab-verify", "start-tab-merge"] {
        assert!(html.contains(step), "missing workflow step {step}");
    }
    assert!(ROOM.contains("snapshot.job?.state==='running'&&snapshot.job.kind==='merge'"));
    assert!(ROOM.contains("room.merge_in_progress"));
    assert!(
        include_str!("../src/ui/startup.css").contains(r#".startup-tabs [role="tab"].running"#)
    );
}
