#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::sidebar;
use eframe::egui::{self, Rect, accesskit::Role};
use serde_json::json;

#[test]
fn sidebar_shows_localized_team_heading_and_cli_authentication_hint() {
    for language in ["en", "ko"] {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(248.0, 800.0),
                )),
                ..Default::default()
            },
            |ui| {
                sidebar::show(&[], &json!({}), language, true, false, ui);
            },
        );
        let labels = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Label || node.role() == Role::Heading)
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>();
        let visible = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        for key in [
            "startup.your_team",
            "startup.register_an_executable_authentication_stays_with_its_cli",
        ] {
            let expected = agentconductor::ui::messages::in_language(language, key);
            assert!(
                labels.contains(&expected) || visible.contains(&expected),
                "{language} missing {expected:?}: labels={labels:?}; visible={visible:?}"
            );
        }
    }
}
