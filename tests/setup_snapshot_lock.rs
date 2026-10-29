use super::unlock_snapshot;
use std::sync::Mutex;

#[test]
fn snapshot_clone_releases_settings_lock_before_rendering() {
    let saved = Mutex::new(String::from("workspace"));
    let snapshot = unlock_snapshot(saved.lock().unwrap());

    assert_eq!(snapshot, "workspace");
    assert!(saved.try_lock().is_ok());
}
