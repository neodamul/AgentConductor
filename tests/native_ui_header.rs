#![cfg(feature = "native-ui-probe")]

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
            "startup.workspace_settings_5ebc59",
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
        assert!(output.shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Rect(rect)
                if rect.fill == context.global_style().visuals.selection.bg_fill
                    && rect.stroke.color == context.global_style().visuals.widgets.hovered.bg_stroke.color)
        }), "the selected step must be visible as a framed tab in {language}");
    }
}

#[test]
fn workspace_step_descriptions_are_one_line_under_each_stage_title() {
    for language in ["en", "ko"] {
        for (step, key) in [
            "startup.saves_the_workspace_name_goal_coordinator_repository_and",
            "startup.role_assignments_first_agent_preferred",
            "startup.generate_a_plan_then_review_each_role_scope",
        ]
        .iter()
        .enumerate()
        {
            let context = egui::Context::default();
            let description = agentconductor::ui::messages::in_language(language, key);
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(860.0, 740.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    header::show(step, language, ui);
                },
            );
            let height = output.shapes.iter().find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == description => {
                    Some(text.galley.size().y)
                }
                _ => None,
            });
            output.textures_delta.set.clear();
            output.textures_delta.free.clear();
            let height = height.unwrap_or_else(|| panic!("missing {description:?} in {language}"));
            assert!(height <= 18.0, "{language} step {step} wraps: {height}px");
        }
    }
}

#[test]
fn all_step_buttons_fit_the_content_column_in_both_languages() {
    for language in ["en", "ko"] {
        for width in [480.0, 836.0] {
            let context = egui::Context::default();
            context.enable_accesskit();
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 580.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    header::show(0, language, ui);
                },
            );
            output.textures_delta.set.clear();
            output.textures_delta.free.clear();
            let buttons = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .map(|(_, node)| node)
                .filter(|node| node.role() == Role::Button)
                .collect::<Vec<_>>();
            assert_eq!(buttons.len(), 3);
            for node in buttons {
                let bounds = node.bounds().unwrap();
                assert!(
                    bounds.x0 >= 0.0 && bounds.x1 <= f64::from(width),
                    "{} exceeds {width}px content: {bounds:?}",
                    node.label().unwrap_or_default()
                );
            }
        }
    }
}
