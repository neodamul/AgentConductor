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
        let output = context.run_ui(Default::default(), |ui| {
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
        let output = context.run_ui(Default::default(), |ui| {
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
        assert!(labels.contains(&expected));
    }
}
