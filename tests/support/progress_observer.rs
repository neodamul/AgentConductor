use super::{MAX_POLL_DELAY, MIN_POLL_DELAY};
use std::time::Duration;

#[test]
fn idle_observation_backoff_is_bounded_and_progress_resets_it() {
    let first = super::next_poll_delay(MIN_POLL_DELAY, false);
    assert_eq!(first, Duration::from_millis(200));
    assert_eq!(
        super::next_poll_delay(MAX_POLL_DELAY, false),
        MAX_POLL_DELAY
    );
    assert_eq!(super::next_poll_delay(MAX_POLL_DELAY, true), MIN_POLL_DELAY);
}
