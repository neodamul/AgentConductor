const WORKFLOW: &str = include_str!("../.github/workflows/platform-validation.yml");
const CHAT_LAUNCHER: &str = include_str!("support/windows_chat_launcher.rs");
const CHAT_SCRIPT: &str = include_str!("support/windows_chat_script.rs");

#[path = "support/windows_job_diagnostic.rs"]
mod job_diagnostic;

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
fn packaged_chat_app_uses_an_interactive_scheduled_task_launcher() {
    assert!(CHAT_LAUNCHER.contains("New-ScheduledTaskPrincipal"));
    assert!(CHAT_LAUNCHER.contains("LogonType Interactive"));
}

#[test]
fn only_a_proven_job_object_breakaway_denial_can_skip_packaged_chat_journeys() {
    assert!(job_diagnostic::host_job_breakaway_denied(
        "Windows denied process breakaway (Access is denied. (os error 5)); this app is inside a host job"
    ));
    assert!(!job_diagnostic::host_job_breakaway_denied(
        "Windows denied process breakaway (Access is denied. (os error 5)); host-job membership unknown"
    ));
    assert!(CHAT_SCRIPT.contains("this app is inside a host job"));
}

#[test]
fn windows_package_workflow_reports_when_host_job_prevents_chat_coverage() {
    let step = WORKFLOW
        .split("name: Verify the packaged Windows default desktop")
        .nth(1)
        .unwrap()
        .split("name: Retain verified package and execution evidence")
        .next()
        .unwrap();
    assert!(step.contains("EXACT_WINDOWS_CHAT_SKIPPED"));
    assert!(step.contains("GITHUB_STEP_SUMMARY"));
    assert!(step.contains("Windows Job Object"));
}
