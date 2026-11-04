//! GitHub #415: the UI is compared with the approved design mock element by element (geometry, computed
//! style and the share of differing pixels) in both themes. Runs tests/design_parity/parity.cjs in headless
//! Chrome and skips when no Chrome is installed (exit code 77 of the runner).
use std::process::Command;

#[test]
fn the_ui_matches_the_approved_design_mock_to_the_pixel() {
    for theme in ["light", "dark"] {
        let output = Command::new("node")
            .arg("tests/design_parity/parity.cjs")
            .env("PARITY_THEME", theme)
            .output()
            .expect("node is required for the UI checks");
        let text = String::from_utf8_lossy(&output.stdout);
        match output.status.code() {
            Some(0) => {}
            Some(77) => eprintln!("skipped: {text}"),
            _ => panic!(
                "{theme}: {text}\n{}",
                String::from_utf8_lossy(&output.stderr)
            ),
        }
    }
}
