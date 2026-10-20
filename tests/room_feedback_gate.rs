//! GitHub #95: the room's feedback gate, exercised through Node like the i18n runtime test.
#[test]
fn send_feedback_is_enabled_for_a_running_task() {
    let result = std::process::Command::new("node")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("tests/room_feedback_gate.cjs")
        .output()
        .expect("Node runs UI behavior tests");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
