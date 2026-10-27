#![cfg(unix)]
//! R-65 / DD-71 / QAS-58: idle pages refresh slowly, pause while hidden, skip icon payloads.
use agentconductor::ui::setup::Startup;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

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
