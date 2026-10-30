#![cfg(feature = "native-ui-probe")]

#[test]
fn metadata_work_uses_fast_refresh_but_idle_profiles_do_not() {
    let running = serde_json::json!({
        "metadata_job": {"kind": "catalog:claude", "state": "running"}
    });
    let idle = serde_json::json!({"metadata_job": {"state": "completed"}});
    assert_eq!(
        agentconductor::native_ui_probe::room::room_refresh_interval(&running),
        std::time::Duration::from_secs(1)
    );
    assert_eq!(
        agentconductor::native_ui_probe::room::room_refresh_interval(&idle),
        std::time::Duration::from_secs(30)
    );
}

#[test]
fn waiting_availability_uses_idle_refresh_interval() {
    let waiting = serde_json::json!({"report":{"state":"waiting_availability"}});
    assert_eq!(
        agentconductor::native_ui_probe::room::room_refresh_interval(&waiting),
        std::time::Duration::from_secs(30)
    );
}

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
