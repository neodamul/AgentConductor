use agentconductor::{
    native_ui_probe::workspace::{show_role_rules, show_role_rules_in_language},
    roles::RoleAgent,
};
use eframe::egui;
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
    for label in ["Role", "agent", "Model", "Alternate"] {
        assert!(
            labels.iter().any(|actual| actual == label),
            "{label}: {labels:?}"
        );
    }
    assert!(labels.iter().any(|label| label == "testing agent"));
    assert!(
        labels
            .iter()
            .any(|label| label == "testing alternate agent")
    );
    assert!(labels.iter().any(|label| label == "testing Model"));
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
        "역할",
        "에이전트",
        "모델",
        "대체 에이전트",
        "검증 에이전트",
        "검증 모델",
        "검증 대체 에이전트",
    ] {
        assert!(
            labels.iter().any(|actual| actual == label),
            "{label}: {labels:?}"
        );
    }
}

#[test]
fn role_table_keeps_all_columns_visible_without_registered_agents() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut roles = BTreeMap::new();
    let mut output = context.run_ui(Default::default(), |ui| {
        show_role_rules_in_language(&mut roles, &[], true, "en", ui)
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
    assert!(labels.contains(&"Role"));
    assert!(labels.contains(&"agent"));
    assert!(labels.contains(&"Alternate"));
}

#[test]
fn suggested_roles_follow_the_existing_cli_preferences() {
    let profiles = [
        serde_json::json!({"id":"claude","parser":"claude"}),
        serde_json::json!({"id":"codex","parser":"codex"}),
        serde_json::json!({"id":"opencode","parser":"opencode"}),
        serde_json::json!({"id":"local","parser":"custom"}),
    ];
    let roles = agentconductor::native_ui_probe::workspace::suggested_role_rules(&profiles);
    let ids = |role: &str| {
        roles[role]
            .iter()
            .map(|entry| entry.agent.as_str())
            .collect::<Vec<_>>()
    };
    assert_eq!(ids("verification"), ["claude", "codex"]);
    assert_eq!(ids("development"), ["codex", "claude"]);
    assert_eq!(ids("testing"), ["opencode", "local"]);
}
