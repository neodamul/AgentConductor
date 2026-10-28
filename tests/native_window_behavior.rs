#[test]
fn macos_picker_is_owned_by_the_app_and_dispatched_to_the_main_thread() {
    let picker = include_str!("../src/ui/picker.rs");
    assert!(
        !picker.contains("osascript"),
        "an external script owns an unrelated dialog"
    );
    let native = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/picker/macos.rs"
    ))
    .unwrap();
    for required in [
        "NSOpenPanel",
        "dispatch_async_f",
        "activateIgnoringOtherApps:",
        "makeKeyAndOrderFront:",
    ] {
        assert!(native.contains(required));
    }
}
#[test]
fn windows_desktop_and_background_children_have_no_console_windows() {
    assert!(include_str!("../src/main.rs").contains("windows_subsystem = \"windows\""));
    assert!(include_str!("../src/process/windows_job.rs").contains("CREATE_NO_WINDOW"));
    assert!(include_str!("../src/git.rs").contains("crate::subprocess::command("));
    assert!(include_str!("../src/messenger/send.rs").contains("crate::subprocess::command("));
    let controller = include_str!("../src/ui/controller.rs");
    assert!(controller.contains("crate::subprocess::background_command("));
    assert!(!controller.contains("std::process::Command::new("));
}
