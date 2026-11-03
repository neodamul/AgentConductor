use agentconductor::ui::messages::{in_language, interpolate};
#[path = "support/catalog.rs"]
mod catalog;

#[test]
fn locale_catalogs_share_ids_and_native_messages_use_the_requested_language() {
    let en = catalog::english();
    let ko = catalog::korean();
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
