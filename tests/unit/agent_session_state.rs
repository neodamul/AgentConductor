use super::session_state;
#[test]
fn a_running_task_in_a_finished_run_is_stale() {
    assert_eq!(session_state("running", "stopped"), "stale");
    assert_eq!(session_state("running", "running"), "running");
    assert_eq!(
        session_state("waiting_approval", "waiting_tool_approval"),
        "waiting"
    );
    assert_eq!(session_state("approved", "completed"), "done");
}
