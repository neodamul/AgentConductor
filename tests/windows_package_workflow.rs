const WORKFLOW: &str = include_str!("../.github/workflows/platform-validation.yml");
const CHAT_LAUNCHER: &str = include_str!("support/windows_chat_launcher.rs");

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

#[test]
fn packaged_chat_app_runs_outside_the_runner_job() {
    assert!(CHAT_LAUNCHER.contains("New-ScheduledTaskPrincipal"));
    assert!(CHAT_LAUNCHER.contains("LogonType Interactive"));
}
