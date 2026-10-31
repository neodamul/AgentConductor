//! GitHub #180: the workspace page embeds the Optimize prompt action, its dialog and script.
use agentconductor::ui::Room;

#[test]
fn the_startup_document_embeds_the_optimize_controls_and_script() {
    let root = std::env::temp_dir().join(format!("ac-optimize-ui-{}", std::process::id()));
    let room = Room::startup(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let document = room.document().unwrap();
    for needle in [
        "id=\"start-optimize-button\"",
        "id=\"start-optimize\"",
        "id=\"start-optimize-use\"",
        "id=\"start-optimize-keep\"",
        "function optimizeUi(",
        "optimizeUi({q,request",
        "optimize.update();sources.update();chat.update();renderStatus();",
        ".optimize-goal{",
    ] {
        assert!(document.contains(needle), "{needle}");
    }
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_optimize_script_requests_shows_applies_and_reports() {
    let result = std::process::Command::new("node")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("tests/optimize_runtime.cjs")
        .output()
        .expect("Node runs UI behavior tests");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
