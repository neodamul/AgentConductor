//! GitHub #691: a held mutex fails the request after the deadline instead of parking it forever.
use agentconductor::bounded_lock::{Unavailable, lock_within};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};
#[test]
fn a_held_lock_fails_after_the_deadline_and_a_free_one_is_taken() {
    let mutex = Mutex::new(1);
    assert_eq!(*lock_within(&mutex, Duration::from_millis(50)).unwrap(), 1);
    let _held = mutex.lock().unwrap();
    let started = Instant::now();
    assert_eq!(
        lock_within(&mutex, Duration::from_millis(100)).err(),
        Some(Unavailable::Held)
    );
    assert!(
        started.elapsed() >= Duration::from_millis(100)
            && started.elapsed() < Duration::from_secs(5)
    );
}
#[test]
fn a_poisoned_lock_is_reported_as_such() {
    let mutex = std::sync::Arc::new(Mutex::new(0));
    let other = mutex.clone();
    let _ = std::thread::spawn(move || {
        let _guard = other.lock().unwrap();
        panic!("poison");
    })
    .join();
    assert_eq!(
        lock_within(&mutex, Duration::from_millis(50)).err(),
        Some(Unavailable::Poisoned)
    );
}
