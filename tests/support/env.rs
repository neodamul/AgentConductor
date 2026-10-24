//! Serialized, restoring changes to the test process environment (DD-187).
#![allow(dead_code)] // shared by many test binaries; each uses a subset
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    sync::{Mutex, MutexGuard},
};

static LOCK: Mutex<()> = Mutex::new(());

/// Holds the environment lock for a whole test; the originals return on drop.
pub struct Env {
    originals: BTreeMap<OsString, Option<OsString>>,
    _lock: MutexGuard<'static, ()>,
}

pub fn lock() -> Env {
    Env {
        originals: BTreeMap::new(),
        // The mutex guards no data, so one failed test must not poison the others.
        _lock: LOCK.lock().unwrap_or_else(|e| e.into_inner()),
    }
}

impl Env {
    pub fn set(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
        self.remember(key.as_ref());
        apply(key.as_ref(), Some(value.as_ref()));
    }
    pub fn remove(&mut self, key: impl AsRef<OsStr>) {
        self.remember(key.as_ref());
        apply(key.as_ref(), None);
    }
    fn remember(&mut self, key: &OsStr) {
        self.originals
            .entry(key.to_owned())
            .or_insert_with(|| std::env::var_os(key));
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        for (key, original) in &self.originals {
            apply(key, original.as_deref());
        }
    }
}

fn apply(key: &OsStr, value: Option<&OsStr>) {
    // SAFETY: every caller holds LOCK through `Env`, so no two helper calls race. shortcut: a
    // non-mutating test thread of the same binary may still read the environment; upgrade to one
    // process per test (for example `cargo nextest`) for full soundness.
    unsafe {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
