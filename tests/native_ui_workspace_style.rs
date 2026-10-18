#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe as native;
use eframe::egui::{self, Color32, accesskit::Role};

#[test]
fn workspace_inputs_have_visible_boundaries_and_room_for_a_goal() {
    for (theme, border) in [
        ("light", Color32::from_rgb(0xc4, 0xc4, 0xcc)),
        ("dark", Color32::from_rgb(0x3a, 0x48, 0x44)),
    ] {
        let context = egui::Context::default();
        context.enable_accesskit();
        native::appearance::apply(theme, &context);
        let mut probe = native::Probe::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1140.0, 740.0),
            )),
            ..Default::default()
        };
        let mut output = context.run_ui(input, |ui| probe.show(ui));
        let input_rects = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect)
                    if rect.stroke.color == border
                        && rect.stroke.width >= 1.0
                        && rect.rect.width() > 180.0
                        && (25.0..=60.0).contains(&rect.rect.height()) =>
                {
                    Some(rect.rect)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let multiline_goal = output.shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Rect(rect)
                if rect.stroke.color == border
                    && rect.stroke.width >= 1.0
                    && rect.rect.width() > 180.0
                    && rect.rect.height() > 60.0)
        });
        let controls = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| matches!(node.role(), Role::TextInput | Role::ComboBox))
            .map(|(_, node)| {
                let bounds = node.bounds().unwrap();
                (node.label().map(str::to_owned), bounds.y1 - bounds.y0)
            })
            .collect::<Vec<_>>();
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        assert!(
            input_rects.len() >= 2,
            "{theme} fields need visible borders"
        );
        assert!(multiline_goal, "{theme} goal must show several lines");
        assert!(!controls.is_empty(), "{theme} needs native form controls");
        assert!(
            controls.iter().all(|(_, height)| *height <= 50.0),
            "{theme} compact form controls are too tall: {controls:?}"
        );
    }
}

#[test]
fn workspace_settings_sharing_heading_has_readable_contrast_in_both_themes() {
    for (theme, background) in [
        ("light", Color32::WHITE),
        ("dark", Color32::from_rgb(0x1a, 0x22, 0x20)),
    ] {
        let context = egui::Context::default();
        native::appearance::apply(theme, &context);
        let mut state = native::workspace::sharing::State::default();
        let mut output = context.run_ui(Default::default(), |ui| {
            native::workspace::sharing::show(&mut state, "en", true, false, ui);
        });
        let foreground = output.shapes.iter().find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "Workspace settings sharing" => text
                .galley
                .job
                .sections
                .first()
                .map(|section| section.format.color)
                .map(|color| {
                    if color == Color32::PLACEHOLDER {
                        text.fallback_color
                    } else {
                        color
                    }
                }),
            _ => None,
        });
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        let foreground = foreground.expect("workspace settings sharing heading is rendered");
        assert!(
            contrast(foreground, background) >= 4.5,
            "{theme} workspace sharing heading contrast is too weak: {foreground:?}"
        );
    }
}

#[test]
fn team_role_labels_have_readable_contrast_in_both_themes() {
    for (theme, background) in [
        ("light", Color32::WHITE),
        ("dark", Color32::from_rgb(0x1a, 0x22, 0x20)),
    ] {
        let context = egui::Context::default();
        context.enable_accesskit();
        native::appearance::apply(theme, &context);
        let mut roles = agentconductor::roles::RoleRules::default();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1140.0, 740.0),
                )),
                ..Default::default()
            },
            |ui| native::workspace::show_role_rules_in_language(&mut roles, &[], true, "en", ui),
        );
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
        for role in [
            "verification",
            "development",
            "documentation",
            "knowledge",
            "testing",
        ] {
            assert!(
                labels.contains(&role),
                "{theme} missing role {role}: {labels:?}"
            );
        }
        assert!(!labels.contains(&"Add agent"));
        assert!(contrast(context.global_style().visuals.text_color(), background) >= 4.5);
    }
}

fn contrast(foreground: Color32, background: Color32) -> f64 {
    fn luminance(color: Color32) -> f64 {
        let [red, green, blue, _] = color.to_array();
        let channels = [red, green, blue]
            .into_iter()
            .map(|channel| {
                let value = f64::from(channel) / 255.0;
                if value <= 0.04045 {
                    value / 12.92
                } else {
                    ((value + 0.055) / 1.055).powf(2.4)
                }
            })
            .collect::<Vec<_>>();
        0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]
    }
    let (light, dark) = (luminance(foreground), luminance(background));
    (light.max(dark) + 0.05) / (light.min(dark) + 0.05)
}

#[test]
fn sidebar_agent_identity_has_readable_contrast_in_both_themes() {
    use agentconductor::native_ui_probe::workspace::sidebar;
    use serde_json::json;

    for (theme, background) in [
        ("light", Color32::WHITE),
        ("dark", Color32::from_rgb(0x1a, 0x22, 0x20)),
    ] {
        let profiles = vec![json!({
            "id":"local-fixture", "parser":"custom", "max_sessions":3
        })];
        let snapshot = json!({
            "config":{"coordinator":"local-fixture","profiles":profiles},
            "quotas":{"local-fixture":{"status":"available"}}
        });
        let context = egui::Context::default();
        native::appearance::apply(theme, &context);
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(260.0, 740.0),
                )),
                ..Default::default()
            },
            |ui| {
                assert_eq!(
                    sidebar::show(&profiles, &snapshot, "en", true, false, ui),
                    None
                )
            },
        );
        let identity = output.shapes.iter().find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "local-fixture" => Some(
                text.galley
                    .job
                    .sections
                    .first()
                    .map_or(text.fallback_color, |section| {
                        if section.format.color == Color32::PLACEHOLDER {
                            text.fallback_color
                        } else {
                            section.format.color
                        }
                    }),
            ),
            _ => None,
        });
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        let foreground = identity.expect("sidebar agent identity is rendered");
        assert!(
            contrast(foreground, background) >= 4.5,
            "{theme} sidebar agent identity contrast is too weak: {foreground:?}"
        );
    }
}
