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
            "startup.what_should_the_team_work_on",
        );
        assert!(
            headings.contains(&title) || labels.contains(&title),
            "the existing page-level goal prompt is missing in {language}; headings={headings:?} labels={labels:?}"
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
        assert!(
            output.shapes.iter().any(|shape| {
                matches!(&shape.shape, egui::Shape::Rect(rect)
                if rect.fill == context.global_style().visuals.selection.bg_fill)
            }),
            "the selected step must be visible as a highlighted tab in {language}"
        );
    }
}

#[test]
fn workspace_header_identifies_the_selected_step_in_both_languages() {
    let steps = [
        "startup.step_1_workspace",
        "startup.step_2_team",
        "startup.step_3_your_execution_decision",
    ];
    for language in ["en", "ko"] {
        for (step, step_key) in steps.iter().enumerate() {
            let context = egui::Context::default();
            let title = agentconductor::ui::messages::in_language(
                language,
                "startup.what_should_the_team_work_on",
            );
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
            let visible = output
                .shapes
                .iter()
                .filter_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let present = output.shapes.iter().any(|shape| {
                matches!(&shape.shape,
                egui::Shape::Text(text) if text.galley.job.text == title)
            });
            let eyebrow = agentconductor::ui::messages::in_language(language, step_key);
            output.textures_delta.set.clear();
            output.textures_delta.free.clear();
            assert!(present, "{language} step {step} lost {title:?}");
            assert!(
                visible.contains(&eyebrow),
                "{language} step {step} should identify itself with {eyebrow:?}; visible={visible:?}"
            );
        }
    }
}

#[test]
fn stage_cards_reuse_the_localized_step_eyebrow() {
    let steps = [
        "startup.step_1_workspace",
        "startup.step_2_team",
        "startup.step_3_your_execution_decision",
    ];
    for language in ["en", "ko"] {
        for (step, key) in steps.iter().enumerate() {
            let context = egui::Context::default();
            let mut output = context.run_ui(Default::default(), |ui| {
                header::show_card_eyebrow(step, language, ui);
            });
            let expected = agentconductor::ui::messages::in_language(language, key);
            assert!(output.shapes.iter().any(|shape| {
                matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == expected)
            }));
            output.textures_delta.set.clear();
            output.textures_delta.free.clear();
        }
    }
}

#[test]
fn invalid_workspace_step_falls_back_to_workspace_heading() {
    for language in ["en", "ko"] {
        let context = egui::Context::default();
        let mut output = context.run_ui(Default::default(), |ui| {
            header::show(usize::MAX, language, ui);
        });
        let expected =
            agentconductor::ui::messages::in_language(language, "startup.step_1_workspace");
        assert!(output.shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == expected)
        }));
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
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
            let bounds = buttons
                .iter()
                .map(|node| node.bounds().unwrap())
                .collect::<Vec<_>>();
            assert!(
                bounds
                    .iter()
                    .all(|rect| (44.0..=56.0).contains(&(rect.y1 - rect.y0))),
                "stage tabs should match the numbered 50px main layout in {language} at {width}px: {bounds:?}"
            );
            let widths = bounds
                .iter()
                .map(|rect| rect.x1 - rect.x0)
                .collect::<Vec<_>>();
            assert!(
                widths.iter().all(|width| (*width - widths[0]).abs() < 1.0),
                "stage tabs should divide the content evenly in {language} at {width}px: {bounds:?}"
            );
            for (node, rect) in buttons.into_iter().zip(bounds) {
                assert!(
                    rect.x0 >= 0.0 && rect.x1 <= f64::from(width),
                    "{} exceeds {width}px content: {rect:?}",
                    node.label().unwrap_or_default()
                );
            }
        }
    }
}
