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
    let mut coordinator = String::new();
    let mut roles = agentconductor::roles::RoleRules::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::workspace::profile_tools::show(
            &mut profiles,
            &mut coordinator,
            &mut roles,
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
    assert!(labels.iter().any(|label| label == "Remove agent claude"));
}

#[test]
fn profile_removal_requires_a_confirmation_screen() {
    let mut profiles = vec![serde_json::json!({"id":"claude","parser":"claude"})];
    let snapshot = serde_json::json!({"config":{"profiles":[{"id":"claude"}]}});
    let mut editor = agentconductor::native_ui_probe::workspace::profile_tools::Editor::default();
    editor.request_remove("claude");
    let mut coordinator = "claude".to_owned();
    let mut roles = agentconductor::roles::RoleRules::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::workspace::profile_tools::show(
            &mut profiles,
            &mut coordinator,
            &mut roles,
            &snapshot,
            true,
            &mut editor,
            ui,
        );
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(labels.contains(&"Confirm remove agent claude"));
    assert!(labels.contains(&"Cancel remove agent claude"));
    assert_eq!(profiles.len(), 1);
}

#[test]
fn removing_a_profile_cleans_local_coordinator_roles_and_fallback_references() {
    use agentconductor::native_ui_probe::workspace::profile_tools::{Editor, remove_profile};
    let mut profiles = vec![
        serde_json::json!({"id":"codex","fallback":{"agent":"claude"}}),
        serde_json::json!({"id":"claude"}),
    ];
    let mut coordinator = "claude".to_owned();
    let mut roles = serde_json::from_value(serde_json::json!({
        "development":[{"agent":"claude"}],
        "testing":[{"agent":"codex"}]
    }))
    .unwrap();
    let mut editor = Editor::default();
    editor.open(&profiles[1]);
    editor.request_remove("claude");
    assert!(remove_profile(
        &mut profiles,
        &mut coordinator,
        &mut roles,
        &mut editor,
        "claude"
    ));
    assert_eq!(profiles.len(), 1);
    assert_eq!(coordinator, "codex");
    assert!(profiles[0]["fallback"].is_null());
    assert!(!roles.contains_key("development"));
    assert_eq!(roles["testing"][0].agent, "codex");
    assert!(!editor.is_editing());
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
        &snapshot,
        &profiles,
        &Default::default(),
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
fn provider_switch_replaces_template_and_generates_reference_rename() {
    let current = serde_json::json!({"id":"company","parser":"custom","executable":"aipro","args":["--old"],"catalog":{"models":["old"]}});
    let template = serde_json::json!({"id":"codex","parser":"codex","executable":"codex","args":["exec","{task_file}"],"input_mode":"file","permission_mode":"native_sandbox","max_sessions":2,"quota_refresh_minutes":5,"catalog":null});
    let profiles = vec![
        current.clone(),
        serde_json::json!({"id":"codex"}),
        serde_json::json!({"id":"reviewer"}),
    ];
    let (switched, renames) =
        agentconductor::native_ui_probe::workspace::profile_tools::switch_profile(
            &current, &template, &profiles,
        )
        .unwrap();
    assert_eq!(switched["id"], "codex-2");
    assert_eq!(switched["parser"], "codex");
    assert_eq!(switched["args"][0], "exec");
    assert!(switched["catalog"].is_null());
    assert_eq!(renames["company"], "codex-2");
    let mut local_profiles = vec![
        serde_json::json!({"id":"codex-2"}),
        serde_json::json!({"id":"helper","fallback":{"agent":"company"}}),
    ];
    let mut coordinator = "company".to_owned();
    let mut roles = serde_json::from_value(serde_json::json!({
        "development":[{"agent":"company"}]
    }))
    .unwrap();
    agentconductor::native_ui_probe::workspace::profile_tools::rename_references(
        &mut local_profiles,
        &mut coordinator,
        &mut roles,
        &renames,
    );
    assert_eq!(coordinator, "codex-2");
    assert_eq!(roles["development"][0].agent, "codex-2");
    assert_eq!(local_profiles[1]["fallback"]["agent"], "codex-2");
    let request = agentconductor::native_ui_probe::workspace::profile_tools::profile_save_request(
        &serde_json::json!({"revision":"r1"}),
        &[switched],
        &renames,
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&request).unwrap()["renames"]["company"],
        "codex-2"
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
fn editor_switches_all_template_fields_before_saving() {
    let current = serde_json::json!({
        "id":"company", "parser":"custom", "executable":"aipro", "args":["--old"],
        "input_mode":"stdin", "max_sessions":1, "quota_refresh_minutes":19,
        "catalog":{"models":["old-model"]}, "environment":{"TOKEN":"private"}
    });
    let template = serde_json::json!({
        "id":"opencode", "parser":"opencode", "executable":"/bin/opencode",
        "args":["run","{task_file}"], "input_mode":"file", "max_sessions":3,
        "quota_refresh_minutes":5, "permission_mode":"unverified", "catalog":null
    });
    let profiles = vec![current.clone(), serde_json::json!({"id":"opencode"})];
    let mut switched = current.clone();
    let mut editor = agentconductor::native_ui_probe::workspace::profile_tools::Editor::default();
    editor.open(&current);
    editor.switch_provider(&template, &profiles);
    editor.apply(&mut switched).unwrap();
    assert_eq!(switched["id"], "opencode-2");
    assert_eq!(switched["parser"], "opencode");
    assert_eq!(switched["executable"], "/bin/opencode");
    assert_eq!(switched["args"][0], "run");
    assert_eq!(switched["input_mode"], "file");
    assert_eq!(switched["max_sessions"], 3);
    assert_eq!(switched["quota_refresh_minutes"], 5);
    assert!(switched["catalog"].is_null());
    assert!(switched["environment"].is_null());
    assert_eq!(editor.renames()["company"], "opencode-2");
}

#[test]
fn executable_resolution_only_updates_the_value_that_requested_it() {
    let mut editor = agentconductor::native_ui_probe::workspace::profile_tools::Editor::default();
    editor
        .open(&serde_json::json!({"id":"local","parser":"custom","executable":"aipro","args":[]}));
    editor.apply_resolution("aipro", Some("/tools/aipro"));
    assert_eq!(editor.executable(), "/tools/aipro");
    editor.apply_resolution("/tools/aipro", Some("/stale/path"));
    editor.set_executable("/operator/choice");
    assert_eq!(editor.executable(), "/operator/choice");
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
        &Default::default(),
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
