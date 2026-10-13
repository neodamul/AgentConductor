//! DD-121/DD-125: each phase holds its own tasks in the plan review.
const STARTUP: &str = include_str!("../src/ui/startup.js");

#[test]
fn editable_tasks_sit_inside_the_phase_under_review() {
    assert!(STARTUP.contains("if(ph.id===reviewing)item.append(...cards)"));
}

#[test]
fn a_proposal_whose_phase_completed_is_not_under_review() {
    assert!(STARTUP.contains("ph.id===proposed&&ph.stage==='completed'"));
    assert!(STARTUP.contains("reviewing=stale?null:proposed"));
    assert!(STARTUP.contains("c.hidden=stale"));
}

#[test]
fn a_click_outside_the_workspace_menu_closes_it() {
    assert!(STARTUP.contains("document.addEventListener('pointerdown'"));
    assert!(STARTUP.contains("q('workspace-list').contains(e.target)"));
}
