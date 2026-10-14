use agentconductor::ui::Room;
#[test]
fn user_manual_is_bundled_localized_and_available_from_the_shared_workspace_menu() {
    let root = std::env::temp_dir().join(format!(
        "ac-manual-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let startup = Room::startup(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let page = startup.document().unwrap();
    assert!(page.contains("start-manual-dialog") && page.contains("AC_MANUAL.open()"));
    assert!(page.contains("const AC_MANUAL_TEXT="));
    assert!(
        page.contains("supports_websockets = false")
            && page.contains("model_providers.openai-https")
    );
    assert!(page.contains("User manual") && page.contains("사용자 매뉴얼"));
    assert!(page.contains("ac-language-change") && !page.contains("manual-body').innerHTML"));
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
