#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::profile_tools::RenderContext;
use eframe::egui::{self, accesskit::Role};

#[test]
fn unqueried_profile_catalog_is_labeled_instead_of_counted_as_zero() {
    for (language, expected) in [("en", "Models not queried"), ("ko", "모델 미조회")] {
        let mut profiles = vec![serde_json::json!({"id":"local","parser":"custom"})];
        let snapshot = serde_json::json!({"config":{"profiles":[{"id":"local"}]}});
        let mut coordinator = String::new();
        let mut roles = agentconductor::roles::RoleRules::default();
        let mut editor =
            agentconductor::native_ui_probe::workspace::profile_tools::Editor::default();
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = context.run_ui(Default::default(), |ui| {
            agentconductor::native_ui_probe::workspace::profile_tools::show_in_language(
                &mut profiles,
                &mut coordinator,
                &mut roles,
                &mut editor,
                RenderContext {
                    snapshot: &snapshot,
                    editable: true,
                    language,
                    show_registry: true,
                },
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
            .filter(|(_, node)| node.role() == Role::Label)
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>();
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        assert!(labels.contains(&expected));
    }
}

#[test]
fn missing_paid_agent_quota_is_visible_and_accessibly_named() {
    for (language, expected) in [("en", "—: Quota unavailable"), ("ko", "—: 사용량 없음")]
    {
        let mut profiles = vec![serde_json::json!({"id":"claude","parser":"claude"})];
        let snapshot = serde_json::json!({"config":{"profiles":[{"id":"claude"}]}});
        let mut coordinator = String::new();
        let mut roles = agentconductor::roles::RoleRules::default();
        let mut editor =
            agentconductor::native_ui_probe::workspace::profile_tools::Editor::default();
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = context.run_ui(Default::default(), |ui| {
            agentconductor::native_ui_probe::workspace::profile_tools::show_in_language(
                &mut profiles,
                &mut coordinator,
                &mut roles,
                &mut editor,
                RenderContext {
                    snapshot: &snapshot,
                    editable: true,
                    language,
                    show_registry: true,
                },
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
            .filter(|(_, node)| node.role() == Role::Label)
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>();
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        assert!(labels.contains(&expected));
    }
}

#[test]
fn quota_badge_and_refresh_button_share_a_compact_group() {
    let mut profiles = vec![serde_json::json!({
        "id":"claude", "parser":"claude", "catalog":{"models":["sonnet"]}
    })];
    let snapshot = serde_json::json!({
        "config":{"profiles":[{"id":"claude"}]},
        "quota_badges":{"claude":{"text":"73%/100%","title":"Weekly quota"}}
    });
    let mut coordinator = String::new();
    let mut roles = agentconductor::roles::RoleRules::default();
    let mut editor = agentconductor::native_ui_probe::workspace::profile_tools::Editor::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 400.0),
            )),
            ..Default::default()
        },
        |ui| {
            agentconductor::native_ui_probe::workspace::profile_tools::show_in_language(
                &mut profiles,
                &mut coordinator,
                &mut roles,
                &mut editor,
                RenderContext {
                    snapshot: &snapshot,
                    editable: true,
                    language: "en",
                    show_registry: true,
                },
                ui,
            );
        },
    );
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let quota = nodes
        .iter()
        .find(|(_, node)| {
            node.role() == Role::Label
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("73%/100%"))
        })
        .and_then(|(_, node)| node.bounds())
        .unwrap();
    let refresh = nodes
        .iter()
        .find(|(_, node)| {
            node.role() == Role::Button && node.label() == Some("Refresh quota for claude")
        })
        .and_then(|(_, node)| node.bounds())
        .unwrap();
    assert!(refresh.height() <= 19.0, "quota refresh should be compact");
    let gap = refresh.x0 - quota.x1;
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        (gap - 5.0).abs() <= 1.0,
        "quota value and refresh must be adjacent: value={quota:?}, button={refresh:?}"
    );
}

#[test]
fn custom_quota_badge_remains_visible_without_cli_refresh_action() {
    let mut profiles = vec![serde_json::json!({"id":"company","parser":"custom"})];
    let snapshot = serde_json::json!({
        "config":{"profiles":[{"id":"company"}]},
        "quota_badges":{"company":{"text":"41%","title":"Company budget"}}
    });
    let mut coordinator = String::new();
    let mut roles = agentconductor::roles::RoleRules::default();
    let mut editor = agentconductor::native_ui_probe::workspace::profile_tools::Editor::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::workspace::profile_tools::show_in_language(
            &mut profiles,
            &mut coordinator,
            &mut roles,
            &mut editor,
            RenderContext {
                snapshot: &snapshot,
                editable: true,
                language: "en",
                show_registry: true,
            },
            ui,
        );
    });
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let badge_visible = nodes
        .iter()
        .any(|(_, node)| node.label() == Some("41%: Company budget"));
    let refresh_visible = nodes.iter().any(|(_, node)| {
        node.role() == Role::Button && node.label() == Some("Refresh quota for company")
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(badge_visible);
    assert!(!refresh_visible);
}
