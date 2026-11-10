//! GitHub #691: a mutex that a request takes must not park the request forever.
use std::sync::{Mutex, MutexGuard, TryLockError};
use std::time::{Duration, Instant};
/// Why the lock could not be taken: another thread kept it past the deadline, or a holder panicked.
#[derive(Debug, PartialEq, Eq)]
pub enum Unavailable {
    Held,
    Poisoned,
}
pub fn lock_within<T>(mutex: &Mutex<T>, wait: Duration) -> Result<MutexGuard<'_, T>, Unavailable> {
    let deadline = Instant::now() + wait;
    loop {
        match mutex.try_lock() {
            Ok(guard) => return Ok(guard),
            Err(TryLockError::Poisoned(_)) => return Err(Unavailable::Poisoned),
            Err(TryLockError::WouldBlock) if Instant::now() >= deadline => {
                return Err(Unavailable::Held);
            }
            Err(TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}
