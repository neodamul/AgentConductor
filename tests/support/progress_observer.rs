use std::time::Duration;

#[test]
fn idle_progress_scans_back_off_and_output_resets_the_delay() {
    let first = super::next_poll_delay(super::MIN_POLL_DELAY, false);
    assert_eq!(first, Duration::from_millis(200));
    assert_eq!(
        super::next_poll_delay(super::MAX_POLL_DELAY, false),
        Duration::from_secs(2)
    );
    assert_eq!(
        super::next_poll_delay(super::MAX_POLL_DELAY, true),
        Duration::from_millis(100)
    );
}
