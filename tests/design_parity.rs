//! GitHub #415: the UI is compared with the approved design mock element by element (geometry, computed
//! style and the share of differing pixels). Runs tests/design_parity/parity.cjs in headless Chrome and
//! skips when no Chrome is installed (exit code 77 of the runner).
use std::process::Command;

#[test]
fn the_ui_matches_the_approved_design_mock_to_the_pixel() {
    let output = Command::new("node")
        .arg("tests/design_parity/parity.cjs")
        .output()
        .expect("node is required for the UI checks");
    let text = String::from_utf8_lossy(&output.stdout);
    match output.status.code() {
        Some(0) => {}
        Some(77) => eprintln!("skipped: {text}"),
        _ => panic!("{text}\n{}", String::from_utf8_lossy(&output.stderr)),
    }
}
