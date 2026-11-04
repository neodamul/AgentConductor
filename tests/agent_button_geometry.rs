//! #424: real browser layout regression for the agent action strip.
#[test]
fn agent_icons_fit_inside_compact_buttons() {
    let output = std::process::Command::new("node")
        .arg("tests/agent_button_geometry.cjs")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Node starts the agent button layout check");
    if output.status.code() == Some(77) {
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        return;
    }
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
