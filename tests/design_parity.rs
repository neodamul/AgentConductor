//! #420: compare every responsive element in both themes and both reference widths.
use std::process::Command;

#[test]
fn the_ui_matches_the_approved_design_mock_to_the_pixel() {
    for theme in ["light", "dark"] {
        for width in [840, 1280] {
            let mut command = Command::new("node");
            command
                .arg("tests/design_parity/parity.cjs")
                .env("PARITY_THEME", theme)
                .env("PARITY_WIDTH", width.to_string());
            if let Ok(dir) = std::env::var("PARITY_OUT") {
                command
                    .arg("--out")
                    .arg(std::path::Path::new(&dir).join(format!("{theme}-{width}")));
            }
            let output = command
                .output()
                .expect("node is required for the UI checks");
            let text = String::from_utf8_lossy(&output.stdout);
            match output.status.code() {
                Some(0) => {}
                Some(77) if std::env::var_os("CI").is_none() => eprintln!("skipped: {text}"),
                _ => panic!(
                    "{theme}/{width}: {text}\n{}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            }
        }
    }
}
