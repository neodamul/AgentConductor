//! #420: compare every responsive element in both themes and both reference widths.
use std::process::Command;

// GitHub #651: the four theme/width cells are independent browser runs, so they are four tests that run in parallel.
fn cell(theme: &str, width: u32) {
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

#[test]
fn the_ui_matches_the_approved_design_mock_light_840() {
    cell("light", 840);
}

#[test]
fn the_ui_matches_the_approved_design_mock_light_1280() {
    cell("light", 1280);
}

#[test]
fn the_ui_matches_the_approved_design_mock_dark_840() {
    cell("dark", 840);
}

#[test]
fn the_ui_matches_the_approved_design_mock_dark_1280() {
    cell("dark", 1280);
}
