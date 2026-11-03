//! #409: behavioral checks against the real assembled webview, without a live agent.
#[test]
fn anchored_choices_work_in_both_themes_and_reference_widths() {
    let result = std::process::Command::new("node")
        .arg("tests/choice_controls/runtime.cjs")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Node starts the webview choice test");
    if result.status.code() == Some(77) {
        eprintln!("{}", String::from_utf8_lossy(&result.stdout));
        return;
    }
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
