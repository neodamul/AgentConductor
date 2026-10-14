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
    assert!(page.contains("start-manual-dialog") && page.contains("AC_MANUAL.open(request)"));
    assert!(
        !page.contains("const AC_MANUAL_TEXT="),
        "manuals are loaded on demand"
    );
    let english = startup
        .dispatch(br#"{"kind":"manual_document","language":"en"}"#)
        .unwrap();
    let korean = startup
        .dispatch(br#"{"kind":"manual_document","language":"ko"}"#)
        .unwrap();
    let page = english["html"].as_str().unwrap();
    assert!(page.contains("<table>") && page.contains("<th>Command</th>"));
    assert!(korean["html"].as_str().unwrap().contains("<table>"));
    assert!(
        startup
            .dispatch(br#"{"kind":"manual_document","language":"../secret"}"#)
            .is_err()
    );
    assert!(
        page.contains("supports_websockets = false")
            && page.contains("model_providers.openai-https")
    );
    assert!(
        startup.document().unwrap().contains("User manual")
            && startup.document().unwrap().contains("사용자 매뉴얼")
    );
    assert!(
        startup.document().unwrap().contains("ac-language-change") && !page.contains("<script>")
    );
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
