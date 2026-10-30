use super::take_poll_pending;
use crate::native_ui_probe::pending::Pending;

#[test]
fn poll_reply_stays_pending_until_foreground_operation_finishes() {
    let mut poll = Some(Pending::BackgroundRefresh);

    assert!(take_poll_pending(Some(Pending::ChatStart), &mut poll).is_none());
    assert!(matches!(poll, Some(Pending::BackgroundRefresh)));
    assert!(matches!(
        take_poll_pending(None, &mut poll),
        Some(Pending::BackgroundRefresh)
    ));
    assert!(poll.is_none());
}

#[test]
fn poll_reply_is_stale_when_a_state_event_arrived_while_it_was_running() {
    assert!(super::poll_reply_invalidated(true));
    assert!(!super::poll_reply_invalidated(false));
}
