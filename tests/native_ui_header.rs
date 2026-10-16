use agentconductor::native_ui_probe::workspace::header;
use eframe::egui::{self, accesskit::Role};

#[test]
fn workspace_header_keeps_the_goal_prompt_and_all_steps_accessible_in_both_languages() {
    for language in ["en", "ko"] {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut first = context.run_ui(Default::default(), |ui| {
            header::show(0, language, ui);
        });
        first.textures_delta.set.clear();
        first.textures_delta.free.clear();
        let mut output = context.run_ui(Default::default(), |ui| {
            header::show(0, language, ui);
        });
        let nodes = &output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes;
        let headings = nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Heading)
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>();
        let labels = nodes
            .iter()
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>();
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();

        let title = agentconductor::ui::messages::in_language(
            language,
            "startup.what_should_the_team_work_on",
        );
        assert!(
            headings.contains(&title) || labels.contains(&title),
            "missing hero heading in {language}; headings={headings:?} labels={labels:?}"
        );
        for (index, key) in [
            "startup.workspace_settings_5ebc59",
            "startup.agent_roles",
            "startup.review_the_plan",
        ]
        .iter()
        .enumerate()
        {
            let expected = format!(
                "{}. {}",
                index + 1,
                agentconductor::ui::messages::in_language(language, key)
            );
            assert!(
                labels.contains(&expected.as_str()),
                "missing {expected:?} in {language}"
            );
        }
    }
}
