#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::take_manual_screenshot_path;
use std::ffi::OsString;

#[test]
fn workspace_screenshot_waits_for_its_initial_snapshot() {
    let path = OsString::from("workspace.png");
    assert_eq!(
        take_manual_screenshot_path(false, Some(path.clone()), false),
        None
    );
    assert_eq!(
        take_manual_screenshot_path(false, Some(path.clone()), true),
        Some(path)
    );
}

#[test]
fn manual_screenshot_export_runs_only_once_and_requires_a_path() {
    assert_eq!(
        take_manual_screenshot_path(true, Some(OsString::from("again.png")), true),
        None
    );
    assert_eq!(take_manual_screenshot_path(false, None, true), None);
}
