const WORKFLOW: &str = include_str!("../.github/workflows/platform-validation.yml");

#[test]
fn packaged_windows_ui_automation_tests_run_serially() {
    let step = WORKFLOW
        .split("name: Verify the packaged Windows default desktop")
        .nth(1)
        .unwrap()
        .split("name: Retain verified package and execution evidence")
        .next()
        .unwrap();
    assert!(step.contains("--test-threads=1"));
}
