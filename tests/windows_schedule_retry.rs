#[allow(dead_code)]
#[path = "support/windows_schedule.rs"]
mod schedule_support;

#[test]
fn retry_schedule_stops_after_the_first_persisted_result() {
    let mut attempts = Vec::new();
    assert!(
        schedule_support::retry_schedule(|attempt| {
            attempts.push(attempt);
            Ok(attempt == 2)
        })
        .unwrap()
    );
    assert_eq!(attempts, [1, 2]);
}

#[test]
fn retry_schedule_stops_after_its_bounded_attempts() {
    let mut attempts = Vec::new();
    assert!(
        !schedule_support::retry_schedule(|attempt| {
            attempts.push(attempt);
            Ok(false)
        })
        .unwrap()
    );
    assert_eq!(attempts, [1, 2, 3]);
}

#[test]
fn retry_schedule_does_not_repeat_hard_driver_errors() {
    let mut attempts = 0;
    let error = schedule_support::retry_schedule(|_| {
        attempts += 1;
        Err("driver failed".into())
    })
    .unwrap_err();
    assert_eq!(error, "driver failed");
    assert_eq!(attempts, 1);
}
