use agentconductor::native_ui_probe::take_manual_screenshot_path;
use std::ffi::OsString;

#[test]
fn first_workspace_frame_can_export_a_manual_screenshot() {
    let path = OsString::from("workspace.png");
    assert_eq!(
        take_manual_screenshot_path(false, Some(path.clone())),
        Some(path)
    );
}

#[test]
fn manual_screenshot_export_runs_only_once_and_requires_a_path() {
    assert_eq!(
        take_manual_screenshot_path(true, Some(OsString::from("again.png"))),
        None
    );
    assert_eq!(take_manual_screenshot_path(false, None), None);
}
