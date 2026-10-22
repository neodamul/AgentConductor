use super::retry_review;
use std::time::{Duration, Instant};

#[tokio::test]
async fn a_brief_room_lease_race_retries_until_the_shared_deadline() {
    let mut calls = 0;
    let result = retry_review(Instant::now() + Duration::from_secs(2), || {
        calls += 1;
        let call = calls;
        async move {
            if call < 3 {
                Err("lease is held: room snapshot".into())
            } else {
                Ok("reviewed")
            }
        }
    })
    .await;
    assert_eq!(result.unwrap(), "reviewed");
    assert_eq!(calls, 3);
}

#[tokio::test]
async fn unrelated_review_errors_are_not_retried() {
    let mut calls = 0;
    let result = retry_review(Instant::now() + Duration::from_secs(2), || {
        calls += 1;
        async { Err::<(), _>("review profile is missing".into()) }
    })
    .await;
    assert_eq!(result.unwrap_err(), "review profile is missing");
    assert_eq!(calls, 1);
}

#[tokio::test]
async fn held_lease_retries_stop_at_the_deadline() {
    let mut calls = 0;
    let result = retry_review(Instant::now() + Duration::from_millis(250), || {
        calls += 1;
        async { Err::<(), _>("lease is held: room snapshot".into()) }
    })
    .await;
    assert!(result.unwrap_err().contains("timed out"));
    assert!(calls >= 2);
}
