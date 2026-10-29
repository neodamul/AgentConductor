//! GitHub #234: only steps of half a second or more are named in the request diagnostics log.
use agentconductor::request_steps::line;
use std::time::Duration;

#[test]
fn only_steps_of_half_a_second_or_more_are_named() {
    assert_eq!(line("lock", Duration::from_millis(499)), None);
    assert_eq!(
        line("lock", Duration::from_millis(500)).as_deref(),
        Some("step name=lock ms=500")
    );
    assert_eq!(
        line("snapshot", Duration::from_secs(7)).as_deref(),
        Some("step name=snapshot ms=7000")
    );
}
