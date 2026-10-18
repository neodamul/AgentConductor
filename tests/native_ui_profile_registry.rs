#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::profile_tools::{
    Editor, RenderContext, show_in_language,
};
use eframe::egui;

#[test]
fn agent_role_screen_uses_the_sidebar_without_repeating_agent_cards() {
    let mut profiles = vec![serde_json::json!({
        "id": "claude",
        "parser": "claude",
        "catalog": {"models": ["sonnet"]}
    })];
    let snapshot = serde_json::json!({
        "config": {"profiles": [{"id": "claude"}]},
        "quota_badges": {"claude": {"text": "73%/100%", "title": "Weekly quota"}}
    });
    let mut editor = Editor::default();
    let mut coordinator = String::new();
    let mut roles = agentconductor::roles::RoleRules::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        show_in_language(
            &mut profiles,
            &mut coordinator,
            &mut roles,
            &mut editor,
            RenderContext {
                snapshot: &snapshot,
                editable: true,
                language: "en",
                show_registry: false,
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
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for duplicate in [
        "Registered agents",
        "Edit agent claude",
        "Remove agent claude",
        "Refresh models for claude",
        "Refresh quota for claude",
    ] {
        assert!(
            !labels.contains(&duplicate),
            "duplicate agent UI: {labels:?}"
        );
    }
}
