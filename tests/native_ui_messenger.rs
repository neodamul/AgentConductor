#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::messenger::{State, save_request, test_request};
use eframe::egui::{self, accesskit::Role};

#[test]
fn messenger_settings_expose_bilingual_fields_and_explicit_test_action() {
    let snapshot = serde_json::json!({
        "revision":"rev-1",
        "config":{"messenger":{"enabled":true,"kind":"slack","events":["completed","blocked"]}}
    });
    let mut state = State::default();
    state.open(&snapshot);
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut first = context.run_ui(Default::default(), |ui| {
        state.show(&snapshot, "ko", true, ui);
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        assert_eq!(state.show(&snapshot, "ko", true, ui), None);
    });
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let labels = |role| {
        nodes
            .iter()
            .filter(|(_, node)| node.role() == role)
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>()
    };
    let inputs = nodes
        .iter()
        .filter(|(_, node)| matches!(node.role(), Role::TextInput | Role::PasswordInput))
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for field in [
        "startup.webhook_url",
        "startup.slack_channel_id",
        "startup.slack_bot_token",
        "startup.approvers_comma_separated_user_ids",
    ] {
        assert!(
            inputs.contains(&agentconductor::ui::messages::in_language("ko", field)),
            "{field}: {inputs:?}"
        );
    }
    for action in ["startup.send_test_message", "startup.save"] {
        assert!(
            labels(Role::Button).contains(&agentconductor::ui::messages::in_language("ko", action))
        );
    }
    assert!(
        labels(Role::CheckBox).contains(&"막힘"),
        "blocked notification should use its Korean label: {:?}",
        labels(Role::CheckBox)
    );
    let webhook = serde_json::json!({
        "revision":"rev-2",
        "config":{"messenger":{"enabled":false,"kind":"webhook","events":[]}}
    });
    state.open(&webhook);
    let mut first = context.run_ui(Default::default(), |ui| {
        state.show(&webhook, "ko", true, ui);
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        state.show(&webhook, "ko", true, ui);
    });
    let inputs = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| {
            matches!(
                node.role(),
                Role::TextInput | Role::MultilineTextInput | Role::PasswordInput
            )
        })
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for field in [
        "startup.message_body_json",
        "startup.relay_url",
        "startup.relay_secret",
    ] {
        assert!(
            inputs.contains(&agentconductor::ui::messages::in_language("ko", field)),
            "{field}: {inputs:?}"
        );
    }
}

#[test]
fn messenger_requests_keep_secrets_local_and_bind_saves_to_workspace_revision() {
    let snapshot = serde_json::json!({"revision":"rev-7"});
    let settings = agentconductor::messenger::Settings::default();
    let save: serde_json::Value = serde_json::from_slice(
        &save_request(
            &snapshot,
            &settings,
            [
                Some("https://hooks.example.test/x".into()),
                Some("xoxb-test".into()),
                None,
                None,
            ],
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(save["kind"], "save_messenger");
    assert_eq!(save["revision"], "rev-7");
    assert_eq!(save["url"], "https://hooks.example.test/x");
    assert_eq!(save["token"], "xoxb-test");
    let test: serde_json::Value = serde_json::from_slice(
        &test_request(&settings, Some("https://hooks.example.test/x".into())).unwrap(),
    )
    .unwrap();
    assert_eq!(test["kind"], "test_messenger");
    assert_eq!(test["url"], "https://hooks.example.test/x");
    assert_eq!(test["token"], serde_json::Value::Null);
}
