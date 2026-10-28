#[test]
fn chat_worker_is_classified_as_a_short_lived_maintenance_command() {
    let main = include_str!("../src/main.rs");
    assert!(
        main.contains("args.first().map(String::as_str) == Some(\"chat\")"),
        "chat workers must not wait for unrelated messenger shutdown work"
    );
}

#[test]
fn ui_workers_are_spawned_outside_the_desktop_job_lifetime() {
    let controller = include_str!("../src/ui/controller.rs");
    let subprocess = include_str!("../src/subprocess.rs");
    assert!(controller.contains("background_command"));
    assert!(controller.contains("label == \"chat\""));
    assert!(controller.contains("PermissionDenied"));
    assert!(controller.contains("crate::subprocess::command(&self.exe)"));
    assert!(subprocess.contains("CREATE_BREAKAWAY_FROM_JOB"));
}
