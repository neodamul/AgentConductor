use super::{Action, State};
use std::time::{Duration, Instant};

#[test]
fn checking_phase_counts_from_request_and_snapshot_resets_silence_only() {
    let now = Instant::now();
    let mut state = State::default();
    state.requested_at(Action::Check, now);
    state.apply_status_at(
        serde_json::json!({"phase":"checking"}),
        now + Duration::from_secs(1),
    );
    assert_eq!(
        state
            .wait_label("en", now + Duration::from_secs(7))
            .as_deref(),
        Some("Elapsed 0:07 · No new state for 0:06")
    );

    state.apply_status_at(
        serde_json::json!({"phase":"checking"}),
        now + Duration::from_secs(7),
    );
    assert_eq!(
        state
            .wait_label("en", now + Duration::from_secs(7))
            .as_deref(),
        Some("Elapsed 0:07")
    );
}

#[test]
fn install_phase_starts_a_new_clock_and_terminal_phase_clears_it() {
    let now = Instant::now();
    let mut state = State::default();
    state.requested_at(Action::Check, now);
    state.apply_status_at(
        serde_json::json!({"phase":"installing"}),
        now + Duration::from_secs(25),
    );
    assert_eq!(
        state
            .wait_label("en", now + Duration::from_secs(27))
            .as_deref(),
        Some("Elapsed 0:02")
    );

    state.apply_status_at(
        serde_json::json!({"phase":"idle"}),
        now + Duration::from_secs(28),
    );
    assert!(
        state
            .wait_label("en", now + Duration::from_secs(29))
            .is_none()
    );
}
