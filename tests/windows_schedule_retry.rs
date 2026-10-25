#[allow(dead_code)]
#[path = "support/windows_schedule.rs"]
mod schedule_support;

#[test]
fn retry_schedule_stops_after_the_first_persisted_result() {
    let mut attempts = 0;
    assert!(
        schedule_support::retry_schedule(|| {
            attempts += 1;
            Ok(attempts == 2)
        })
        .unwrap()
    );
    assert_eq!(attempts, 2);
}

#[test]
fn retry_schedule_stops_after_its_bounded_attempts() {
    let mut attempts = 0;
    assert!(
        !schedule_support::retry_schedule(|| {
            attempts += 1;
            Ok(false)
        })
        .unwrap()
    );
    assert_eq!(attempts, 3);
}

#[test]
fn retry_schedule_does_not_repeat_hard_driver_errors() {
    let mut attempts = 0;
    let error = schedule_support::retry_schedule(|| {
        attempts += 1;
        Err("driver failed".into())
    })
    .unwrap_err();
    assert_eq!(error, "driver failed");
    assert_eq!(attempts, 1);
}
