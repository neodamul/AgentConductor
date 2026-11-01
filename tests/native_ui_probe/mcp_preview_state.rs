use super::editor::State;

#[test]
fn preview_waiting_for_goal_save_is_consumed_once() {
    let mut state = State::default();
    state.queue_preview_after_save();

    assert!(state.take_preview_after_save());
    assert!(!state.take_preview_after_save());
}

#[test]
fn failed_goal_save_cancels_preview_and_shows_failure() {
    let mut state = State::default();
    state.queue_preview_after_save();
    state.cancel_preview_after_save("save failed");

    assert!(!state.take_preview_after_save());
    assert!(!state.previewing());
}
