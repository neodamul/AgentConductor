//! Compact task rows in the Agent Room. User requests 2026-10-14 (R-128 / DD-137 amendment): the
//! task name leads; a small icon, the agent name and a compact status tag share the next line.
const ROOM: &str = include_str!("../src/ui/room.js");
const CSS: &str = include_str!("../src/ui/room.css");
const RUN_CSS: &str = include_str!("../src/ui/run_screen.css");

#[test]
fn the_task_name_leads_and_icon_name_and_status_share_one_line() {
    // GitHub #675: each row is the mock's outline task: the agent mark, "T1 summary" with a status badge, a one-line scope, a chevron.
    assert!(ROOM.contains(
        "title.append(node('code',session.id,'task-number'),' ',taskLine(plan.prompt)||'')"
    ));
    assert!(ROOM.contains("head.append(title,badge)"));
    assert!(ROOM.contains("button.append(symbol,copy,chevron())"));
    assert!(RUN_CSS.contains("#rail-room .outline-task{display:flex;align-items:center;gap:7px"));
    assert!(RUN_CSS.contains("#rail-room .outline-task .session-symbol.outline-agent"));
}

#[test]
fn brand_marks_sit_in_a_bordered_container_with_the_specified_mark_size() {
    // GitHub #406 / visual-spec: task avatar 28px with a 20px mark, agent avatar 29px with a 19px mark,
    // both inside the 1.5px border (28 = 1.5 + 2.5 + 20 + 2.5 + 1.5; 29 = 1.5 + 3.5 + 19 + 3.5 + 1.5).
    assert!(RUN_CSS.contains("#rail-room .outline-task .app-icon.session-symbol.outline-agent{padding:2.5px;background:var(--fill)}"));
    assert!(CSS.contains(".app-icon.agent-symbol{width:29px;height:29px;box-sizing:border-box;border:1.5px solid var(--line);padding:3.5px;background:var(--fill)}"));
}

#[test]
fn the_avatar_border_carries_the_task_status() {
    // GitHub #400: the avatar carries the status as a 1.5px border and only a running one pulses.
    assert!(
        ROOM.contains("symbol.classList.add('outline-agent','st-'+statusLight(session.status))")
    );
    assert!(ROOM.contains("if(session.status==='running')symbol.classList.add('running')"));
    assert!(!ROOM.contains("agent-light"), "the separate dot is gone");
    assert!(!CSS.contains("agent-light"), "and so are its rules");
    for (name, color) in [("green", "ok"), ("yellow", "warn"), ("red", "err")] {
        let rule = format!(
            "#rail-room .outline-task .session-symbol.outline-agent.st-{name}{{border-color:var(--{color})}}"
        );
        assert!(RUN_CSS.contains(&rule), "{rule}");
    }
    assert!(CSS.contains("border:1.5px solid"));
    assert!(CSS.contains(".session-symbol.running{animation:agent-pulse"));
    assert!(RUN_CSS.contains("prefers-reduced-motion:reduce){#rail-room .outline-task .session-symbol.running{animation:none}"));
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

#[test]
fn task_rows_are_quiet_rows_with_a_trailing_chevron_and_a_selected_accent_edge() {
    // GitHub #410 / behavior.md: separator, hover fill, selected accent edge and a 12px trailing chevron.
    assert!(
        CSS.contains(".session{border:0;border-bottom:1px solid var(--line);border-radius:4px;")
    );
    assert!(CSS.contains(".session:hover{background:var(--fill)}"));
    assert!(
        CSS.contains(
            ".session.active:not([class*=\"attn-\"]){box-shadow:inset 2px 0 var(--accent)}"
        )
    );
    assert!(CSS.contains(
        ".session::after{content:\"\";flex:none;align-self:center;width:12px;height:12px;"
    ));
}

#[test]
fn blocked_and_refused_task_tags_are_red_like_the_avatar_border() {
    // behavior.md: red for blocked/refused, yellow for waiting; the tag tone and the avatar light agree.
    assert!(ROOM.contains("failed:'bad',blocked:'bad',permission_denied:'bad',rejected:'bad',"));
    assert!(ROOM.contains("['failed','blocked','permission_denied'].includes(status)?'red'"));
}

#[test]
fn selecting_a_task_keeps_the_chosen_subview() {
    // behavior.md: selecting a task updates the centre without losing the chosen subview (activity, changes, logs, lessons).
    let start = ROOM.find("function selectTask(index)").unwrap();
    let end = ROOM[start..].find("function selectCoordinator").unwrap();
    assert!(!ROOM[start..start + end].contains("changeTab("));
}
