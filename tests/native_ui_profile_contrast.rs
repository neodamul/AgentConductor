#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe as native;
use eframe::egui::{self, Color32};
use serde_json::json;

#[test]
fn registered_profile_names_have_readable_contrast_in_both_themes() {
    for (theme, background) in [
        ("light", Color32::WHITE),
        ("dark", Color32::from_rgb(0x1a, 0x22, 0x20)),
    ] {
        let mut profiles = vec![json!({"id":"local-fixture","parser":"custom"})];
        let snapshot = json!({"config":{"profiles":[{"id":"local-fixture"}]}});
        let mut coordinator = "local-fixture".to_owned();
        let mut roles = agentconductor::roles::RoleRules::default();
        let mut editor = native::workspace::profile_tools::Editor::default();
        let context = egui::Context::default();
        native::appearance::apply(theme, &context);
        let mut output = context.run_ui(Default::default(), |ui| {
            native::workspace::profile_tools::show(
                &mut profiles,
                &mut coordinator,
                &mut roles,
                &snapshot,
                true,
                &mut editor,
                ui,
            );
        });
        let foreground = output.shapes.iter().find_map(|shape| match &shape.shape {
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
        let foreground = foreground.expect("registered profile name is rendered");
        assert!(
            contrast(foreground, background) >= 4.5,
            "{theme} registered profile name contrast is too weak: {foreground:?}"
        );
    }
}

fn contrast(foreground: Color32, background: Color32) -> f64 {
    fn luminance(color: Color32) -> f64 {
        let [red, green, blue, _] = color.to_array();
        [red, green, blue]
            .into_iter()
            .map(|channel| {
                let value = f64::from(channel) / 255.0;
                if value <= 0.04045 {
                    value / 12.92
                } else {
                    ((value + 0.055) / 1.055).powf(2.4)
                }
            })
            .zip([0.2126, 0.7152, 0.0722])
            .map(|(channel, weight)| channel * weight)
            .sum()
    }
    let (light, dark) = (luminance(foreground), luminance(background));
    (light.max(dark) + 0.05) / (light.min(dark) + 0.05)
}

#[test]
fn compact_quota_value_uses_readable_text_in_both_themes() {
    for (theme, background) in [
        ("light", Color32::WHITE),
        ("dark", Color32::from_rgb(0x1a, 0x22, 0x20)),
    ] {
        let profiles = vec![json!({"id":"claude","parser":"claude"})];
        let snapshot = json!({
            "config":{"profiles":[{"id":"claude"}]},
            "quota_badges":{"claude":{"text":"73%/100%","title":"Weekly quota"}}
        });
        let context = egui::Context::default();
        native::appearance::apply(theme, &context);
        let mut output = context.run_ui(Default::default(), |ui| {
            native::workspace::sidebar::show(&profiles, &snapshot, "en", true, false, ui);
        });
        let foreground = output.shapes.iter().find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "73%/100%" => Some(
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
        let foreground = foreground.expect("quota value is shown in the agent card");
        assert!(
            contrast(foreground, background) >= 4.5,
            "{theme} quota contrast is too weak: {foreground:?}"
        );
    }
}
