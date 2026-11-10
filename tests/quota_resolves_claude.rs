//! GitHub #696: the Claude usage probe runs the CLI under `script`, which searches the app's own PATH; the app opened from Finder has only
//! /usr/bin:/bin, so the probe must hand `script` the resolved path (the same discovery the rest of the app uses).
#[test]
fn the_claude_usage_probe_starts_the_resolved_executable() {
    let source = std::fs::read_to_string("src/quota/terminal.rs").unwrap();
    let start = source
        .find("Command::new(\"/usr/bin/script\")")
        .expect("the script probe");
    let before = &source[..start];
    assert!(
        before.contains("crate::executable::resolve(&profile.executable)"),
        "the executable is resolved before script starts it"
    );
    assert!(
        source[start..].contains("&executable.to_string_lossy()"),
        "script receives the resolved path, not the bare name"
    );
    // The resolver itself knows the Homebrew and per-user folders a minimal PATH lacks.
    let resolver = std::fs::read_to_string("src/executable.rs").unwrap();
    assert!(resolver.contains("/opt/homebrew/bin") && resolver.contains(".local/bin"));
}
