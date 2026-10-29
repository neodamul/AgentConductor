#![cfg(feature = "native-ui-probe")]

#[test]
fn catalog_refresh_preserves_unsaved_profiles_and_does_not_apply_stale_catalogs() {
    let mut local = vec![
        serde_json::json!({"id":"claude","parser":"claude","executable":"claude","name":"unsaved"}),
        serde_json::json!({"id":"new-agent","parser":"custom","executable":"aipro"}),
    ];
    let saved = vec![serde_json::json!({
        "id":"claude","parser":"claude","executable":"claude",
        "catalog":{"models":["observed"]}
    })];
    agentconductor::native_ui_probe::workspace::profile_tools::merge_catalogs(&mut local, &saved);
    assert_eq!(local[0]["name"], "unsaved");
    assert_eq!(local[0]["catalog"]["models"][0], "observed");
    assert_eq!(local[1]["id"], "new-agent");
    assert!(local[1]["catalog"].is_null());
}
