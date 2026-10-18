use agentconductor::{
    native_ui_probe::workspace::{show_role_rules, show_role_rules_in_language},
    roles::RoleAgent,
};
use eframe::egui::{self, accesskit::Role};
use std::collections::BTreeMap;

#[test]
fn native_role_controls_show_only_registered_agents_and_observed_models() {
    let profiles = vec![serde_json::json!({
        "id": "codex",
        "catalog": {"models": ["gpt-observed"]}
    })];
    let mut rules = BTreeMap::from([(
        "testing".into(),
        vec![RoleAgent {
            agent: "codex".into(),
            model: Some("gpt-observed".into()),
        }],
    )]);
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        show_role_rules(&mut rules, &profiles, true, ui)
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(labels.iter().any(|label| label == "testing agent"));
    assert!(labels.iter().any(|label| label == "testing model"));
}

#[test]
fn native_role_controls_localize_korean_labels_and_accessible_names() {
    let profiles = vec![serde_json::json!({
        "id": "codex",
        "catalog": {"models": ["gpt-observed"]}
    })];
    let mut rules = BTreeMap::from([(
        "verification".into(),
        vec![RoleAgent {
            agent: "codex".into(),
            model: None,
        }],
    )]);
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        show_role_rules_in_language(&mut rules, &profiles, true, "ko", ui)
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    for label in [
        "검증",
        "에이전트 추가",
        "검증 에이전트",
        "검증 모델",
        "삭제",
    ] {
        assert!(
            labels.iter().any(|actual| actual == label),
            "{label}: {labels:?}"
        );
    }
}

#[test]
fn role_add_actions_are_disabled_without_an_unassigned_registered_agent() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut roles = BTreeMap::new();
    let mut output = context.run_ui(Default::default(), |ui| {
        show_role_rules_in_language(&mut roles, &[], true, "en", ui)
    });
    let add_buttons = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button && node.label() == Some("Add agent"))
        .map(|(_, node)| node.is_disabled())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert_eq!(add_buttons.len(), 5);
    assert!(add_buttons.iter().all(|disabled| *disabled));
}
