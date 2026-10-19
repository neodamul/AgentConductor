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
        show_role_rules_in_language(&mut rules, &profiles, true, "ko", ui);
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
        show_role_rules_in_language(&mut roles, &[], true, "en", ui);
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
fn role_table_headers_use_theme_foreground_in_both_themes() {
    let mut missing = Vec::new();
    for theme in ["light", "dark"] {
        let context = egui::Context::default();
        agentconductor::native_ui_probe::appearance::apply(theme, &context);
        let expected = if theme == "light" {
            egui::Color32::from_rgb(0x1c, 0x1c, 0x1e)
        } else {
            egui::Color32::from_rgb(0xdd, 0xe6, 0xe1)
        };
        let mut roles = BTreeMap::new();
        let mut output = context.run_ui(Default::default(), |ui| {
            show_role_rules_in_language(&mut roles, &[], true, "en", ui);
        });
        for header in ["Role", "agent", "Model", "Alternate"] {
            let visible = output.shapes.iter().any(|clipped| {
                    matches!(
                        &clipped.shape,
                        egui::epaint::Shape::Text(text)
                            if text.galley.job.text == header
                                && text.galley.num_vertices > 0
                                && clipped.clip_rect.contains_rect(text.visual_bounding_rect())
                                && text.galley.job.sections.iter().all(|section| section.format.color == expected)
                    )
                });
            if !visible {
                missing.push(format!("{theme} theme header {header:?}"));
            }
        }
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
    }
    assert!(
        missing.is_empty(),
        "headers missing theme foreground: {missing:?}"
    );
}

#[test]
fn role_table_fits_a_standard_desktop_content_width() {
    let context = egui::Context::default();
    let bounds = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(820.0, 600.0));
    let mut roles = BTreeMap::new();
    let mut role_table = None;
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(bounds),
            ..Default::default()
        },
        |ui| {
            role_table = Some(show_role_rules_in_language(&mut roles, &[], true, "ko", ui));
        },
    );
    let content = role_table.unwrap();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        content.right() <= bounds.right(),
        "role table overflowed: content right={}, available right={}",
        content.right(),
        bounds.right()
    );
}

#[test]
fn role_table_truncates_long_agent_and_model_ids_to_fit() {
    let context = egui::Context::default();
    let bounds = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(820.0, 600.0));
    let agent = "company-internal-agent-with-a-long-id";
    let model = "company/model-with-a-very-long-versioned-name";
    let alternate = "backup-agent-with-a-long-id";
    let alternate_model = "local/model-with-a-long-name-and-build-number";
    let profiles = vec![
        serde_json::json!({"id": agent, "catalog": {"models": [model]}}),
        serde_json::json!({"id": alternate, "catalog": {"models": [alternate_model]}}),
    ];
    let mut roles = BTreeMap::from([(
        "development".into(),
        vec![
            RoleAgent {
                agent: agent.into(),
                model: Some(model.into()),
            },
            RoleAgent {
                agent: alternate.into(),
                model: Some(alternate_model.into()),
            },
        ],
    )]);
    let mut role_table = None;
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(bounds),
            ..Default::default()
        },
        |ui| {
            role_table = Some(show_role_rules_in_language(
                &mut roles, &profiles, true, "en", ui,
            ));
        },
    );
    let rect = role_table.unwrap();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        rect.right() <= bounds.right(),
        "role table overflowed with long ids: content right={}, available right={}",
        rect.right(),
        bounds.right()
    );
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
