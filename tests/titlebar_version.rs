use agentconductor::ui::Room;

#[test]
fn workspace_and_embedded_room_show_the_executing_build_version() {
    let root = std::env::temp_dir().join(format!(
        "ac-titlebar-version-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let room = Room::startup(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let document = room.document().unwrap();
    let expected = format!(
        "<strong>AgentConductor</strong><span class=\"app-version\">v{}</span>",
        env!("CARGO_PKG_VERSION")
    );
    assert_eq!(document.matches(&expected).count(), 2);
    assert!(!document.contains("{{APP_VERSION}}"));
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
