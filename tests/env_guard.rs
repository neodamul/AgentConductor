//! R-176 / DD-187 / QAS-173: tests change the environment only through a serialized, restoring guard.
#[path = "support/env.rs"]
mod test_env;
use std::{
    sync::{
        Arc, Barrier,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

const KEY: &str = "AGENTCONDUCTOR_ENV_GUARD_TEST";

#[test]
fn values_are_set_and_originals_return_when_the_guard_drops() {
    let mut guard = test_env::lock();
    let original = std::env::var_os(KEY);
    guard.set(KEY, "first");
    guard.set(KEY, "second");
    assert_eq!(std::env::var(KEY).unwrap(), "second");
    guard.remove(KEY);
    assert!(std::env::var_os(KEY).is_none());
    drop(guard);
    let _guard = test_env::lock();
    assert_eq!(std::env::var_os(KEY), original);
}

#[test]
fn a_second_holder_waits_until_the_first_has_restored() {
    let mut guard = test_env::lock();
    guard.set(KEY, "held");
    let entered = Arc::new(AtomicBool::new(false));
    let barrier = Arc::new(Barrier::new(2));
    let waiter = {
        let (entered, barrier) = (entered.clone(), barrier.clone());
        std::thread::spawn(move || {
            barrier.wait();
            let _second = test_env::lock();
            entered.store(true, Ordering::SeqCst);
            std::env::var_os(KEY)
        })
    };
    barrier.wait();
    std::thread::sleep(Duration::from_millis(100));
    assert!(
        !entered.load(Ordering::SeqCst),
        "the lock is held for the whole test"
    );
    drop(guard);
    assert!(
        waiter.join().unwrap().is_none(),
        "the waiter sees the restored environment"
    );
}

#[test]
fn a_panicking_test_restores_and_does_not_poison_the_lock() {
    let result = std::thread::spawn(|| {
        let mut guard = test_env::lock();
        guard.set(KEY, "lost");
        panic!("test failure");
    })
    .join();
    assert!(result.is_err());
    let guard = test_env::lock();
    assert!(std::env::var_os(KEY).is_none());
    drop(guard);
}
