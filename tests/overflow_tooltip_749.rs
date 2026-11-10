#[test]
fn truncated_text_and_screen_tooltips_are_checked() {
    for script in [
        "tests/overflow_tooltip_749.cjs",
        "tests/overflow_screens_749.cjs",
    ] {
        let status = std::process::Command::new("node")
            .arg(script)
            .status()
            .expect("node is required");
        assert!(status.success(), "{script} failed");
    }
}
