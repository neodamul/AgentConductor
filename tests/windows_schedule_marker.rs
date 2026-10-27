#[test]
fn schedule_driver_checks_the_marker_emitted_by_its_script() {
    let source = include_str!("windows_schedule.rs");
    let script = include_str!("support/windows_schedule_script.rs");
    const MARKER: &str = "SCHEDULE_ACTION_SUBMITTED";
    assert!(script.contains(&format!("Write-Output '{MARKER}'")));
    assert!(source.contains(&format!("stdout.contains(\"{MARKER}\")")));
}
