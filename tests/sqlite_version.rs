//! GitHub #979: SQLite 3.51 took the unix VFS big lock inside unixLock while holding the
//! inode mutex, the reverse of open/close, and froze every UI request thread for hours.
#[test]
fn bundled_sqlite_has_the_unix_vfs_lock_order_fix() {
    assert!(
        rusqlite::version_number() >= 3_054_000,
        "{}",
        rusqlite::version()
    );
}
