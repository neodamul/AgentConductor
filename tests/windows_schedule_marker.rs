#[test]
fn schedule_driver_checks_the_marker_emitted_by_its_script() {
    let source = include_str!("windows_schedule.rs");
    const MARKER: &str = "SCHEDULE_ACTION_INPUT_SENT";
    assert!(source.contains(&format!("Write-Output \"{MARKER};")));
    assert!(source.contains(&format!("last_output.contains(\"{MARKER}\")")));
}
