use agentconductor::native_ui_probe::workspace::profile_tools::{Event, metadata_request};
use eframe::egui::{self, accesskit::Role};

#[test]
fn metadata_requests_bind_profile_and_current_revision() {
    let request = metadata_request(&Event::Quota("claude".into()), "revision-7");
    let value: serde_json::Value = serde_json::from_slice(&request).unwrap();
    assert_eq!(value["kind"], "quota");
    assert_eq!(value["agent"], "claude");
    assert_eq!(value["revision"], "revision-7");
}

#[test]
fn native_team_profile_actions_are_accessible_and_show_quota_and_catalog() {
    let mut profiles = vec![serde_json::json!({
        "id": "claude",
        "parser": "claude",
        "catalog": {"models": ["sonnet"]}
    })];
    let snapshot = serde_json::json!({
        "config": {"profiles": [{"id": "claude"}]},
        "quota_badges": {"claude": {"text": "73%/100%", "title": "Weekly quota"}}
    });
    let mut editor = Default::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::workspace::profile_tools::show(
            &mut profiles,
            &snapshot,
            true,
            &mut editor,
            ui,
        );
    });
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let labels = nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(labels.iter().any(|label| label == "Edit agent claude"));
    assert!(
        labels
            .iter()
            .any(|label| label == "Refresh models for claude")
    );
    assert!(
        labels
            .iter()
            .any(|label| label == "Refresh quota for claude")
    );
}

#[test]
fn profile_save_request_updates_only_the_profile_registry() {
    let snapshot = serde_json::json!({
        "revision": "revision-8",
        "config": {"goal": "unsaved goal"}
    });
    let profiles = vec![serde_json::json!({
        "id": "claude",
        "parser": "claude",
        "executable": "/tools/claude",
        "quota_refresh_minutes": 10
    })];
    let request = agentconductor::native_ui_probe::workspace::profile_tools::profile_save_request(
        &snapshot, &profiles,
    );
    let value: serde_json::Value = serde_json::from_slice(&request).unwrap();
    assert_eq!(value["kind"], "save_profiles");
    assert_eq!(value["revision"], "revision-8");
    assert_eq!(value["profiles"][0]["quota_refresh_minutes"], 10);
    assert!(
        value.get("config").is_none(),
        "do not save unrelated workspace drafts"
    );
}

#[test]
fn executable_edit_invalidates_observed_models_and_keeps_runtime_preferences() {
    let mut profile = serde_json::json!({
        "id":"claude", "executable":"claude", "args":["-p"],
        "input_mode":"stdin", "max_sessions":2, "quota_refresh_minutes":5,
        "catalog":{"models":["old-model"]}
    });
    let mut editor = agentconductor::native_ui_probe::workspace::profile_tools::Editor::default();
    editor.open(&profile);
    editor.set_executable("/tools/claude");
    editor.apply(&mut profile).unwrap();
    assert_eq!(profile["executable"], "/tools/claude");
    assert!(profile["catalog"].is_null());
    assert_eq!(profile["max_sessions"], 2);
    assert_eq!(profile["quota_refresh_minutes"], 5);
}

#[test]
fn native_profile_save_persists_runtime_preferences_through_room_dispatch() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-native-profile-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let room = agentconductor::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap();
    let before = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let mut profile = before["templates"][0].clone();
    profile["quota_refresh_minutes"] = 10.into();
    let request = agentconductor::native_ui_probe::workspace::profile_tools::profile_save_request(
        &before,
        &[profile],
    );
    assert_eq!(room.dispatch(&request).unwrap()["saved"], true);
    let after = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(after["config"]["profiles"][0]["quota_refresh_minutes"], 10);
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

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
        std::time::Duration::from_secs(5)
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
