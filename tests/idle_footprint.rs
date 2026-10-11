#![cfg(unix)]
//! R-65 / DD-71 / QAS-58: idle pages refresh slowly, pause while hidden, skip icon payloads.
use agentconductor::ui::setup::Startup;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const STARTUP: &str = include_str!("../src/ui/startup.js");
const ROOM: &str = include_str!("../src/ui/room.js");

#[test]
fn both_pages_use_adaptive_refresh_and_pause_when_hidden() {
    for (name, js) in [("startup", STARTUP), ("room", ROOM)] {
        assert!(
            !js.contains("setInterval(poll,1000)"),
            "{name} still polls every second"
        );
        assert!(
            js.contains("document.hidden") && js.contains("visibilitychange"),
            "{name} must pause while hidden"
        );
        assert!(
            js.contains("5000") && js.contains("1000"),
            "{name} uses fast and idle intervals"
        );
        assert!(
            js.contains("icons:"),
            "{name} asks for icons only when needed"
        );
    }
}

#[test]
fn startup_snapshots_omit_icons_unless_requested() {
    let (root, _) = support::setup("shared");
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let plain = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert!(
        plain.get("icons").is_none(),
        "routine snapshots carry no icon payload"
    );
    let with = startup
        .dispatch(br#"{"kind":"snapshot","icons":true}"#)
        .unwrap();
    assert!(with["icons"].is_object());
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
