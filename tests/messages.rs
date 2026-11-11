use agentconductor::ui::messages::{in_language, interpolate};

#[test]
fn locale_catalogs_share_ids_and_native_messages_use_the_requested_language() {
    let en: serde_json::Value =
        serde_json::from_str(include_str!("../src/ui/messages/en.json")).unwrap();
    let ko: serde_json::Value =
        serde_json::from_str(include_str!("../src/ui/messages/ko.json")).unwrap();
    let en = en.as_object().unwrap();
    let ko = ko.as_object().unwrap();
    assert_eq!(en.keys().collect::<Vec<_>>(), ko.keys().collect::<Vec<_>>());
    assert_eq!(
        in_language("en", "native.enter_a_workspace_name"),
        "enter a workspace name"
    );
    assert_eq!(
        in_language("ko", "native.enter_a_workspace_name"),
        "워크스페이스 이름을 입력하세요"
    );
    assert_eq!(
        in_language("unsupported", "native.enter_a_workspace_name"),
        "enter a workspace name"
    );
    assert_eq!(in_language("ko", "missing.message"), "missing.message");
}

#[test]
fn interpolation_preserves_supplied_values_and_incomplete_tokens() {
    assert_eq!(
        interpolate(
            "{first} / {second}",
            &[("first", "{second}<script>"), ("second", "한글")]
        ),
        "{second}<script> / 한글"
    );
    assert_eq!(interpolate("{missing} {open", &[]), "{missing} {open");
    assert_eq!(
        interpolate("plain text", &[("unused", "data")]),
        "plain text"
    );
}

#[test]
fn javascript_localization_preserves_user_content_when_switching_languages() {
    let result = std::process::Command::new("node")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("tests/i18n_runtime.cjs")
        .output()
        .expect("Node runs UI behavior tests");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn the_native_window_caption_names_no_single_stage() {
    // GitHub #772: the unified window showed "Agent Room" while Workspace settings was on screen.
    for messages in [
        include_str!("../src/ui/messages/en.json"),
        include_str!("../src/ui/messages/ko.json"),
    ] {
        let messages: serde_json::Value = serde_json::from_str(messages).unwrap();
        assert_eq!(messages["native.window_title"], "AgentConductor");
        assert!(messages.get("native.agentconductor_agent_room").is_none());
    }
    for source in [
        include_str!("../src/ui/native/cocoa.rs"),
        include_str!("../src/ui/native/webview2.rs"),
    ] {
        assert!(source.contains("\"native.window_title\""));
    }
}
