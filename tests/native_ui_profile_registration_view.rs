#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::{
    appearance,
    workspace::profile_tools::{self, Editor},
};
use eframe::egui::{self, Color32, accesskit::Role};
use serde_json::json;

#[test]
fn registration_heading_and_empty_fallback_are_visible_in_both_locales_and_themes() {
    for language in ["en", "ko"] {
        for theme in ["light", "dark"] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            appearance::apply(theme, &ctx);
            let mut profiles = Vec::new();
            let mut coordinator = String::new();
            let mut roles = Default::default();
            let mut editor = Editor::default();
            editor.open_new(
                &json!({"id":"local-agent","parser":"custom","executable":"aipro","args":[]}),
            );
            let mut output = None;
            for _ in 0..2 {
                let mut rendered = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1140.0, 772.0),
                        )),
                        ..Default::default()
                    },
                    |ui| {
                        profile_tools::show_in_language(
                            &mut profiles,
                            &mut coordinator,
                            &mut roles,
                            &mut editor,
                            profile_tools::RenderContext {
                                snapshot: &json!({}),
                                editable: true,
                                language,
                                show_registry: false,
                            },
                            ui,
                        );
                    },
                );
                rendered.textures_delta.set.clear();
                rendered.textures_delta.free.clear();
                output = Some(rendered);
            }
            let output = output.unwrap();
            let heading =
                agentconductor::ui::messages::in_language(language, "startup.register_cli_agent");
            let no_fallback = if language == "ko" { "없음" } else { "None" };
            assert!(
                output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.role() == Role::ComboBox
                        && node.value() == Some(no_fallback)),
                "{language} fallback value must be translated"
            );
            let shape = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.job.text == heading => {
                        Some((shape.clip_rect, text))
                    }
                    _ => None,
                })
                .expect("registration must have its own heading");
            let color = shape.1.galley.job.sections[0].format.color;
            let color = if color == Color32::PLACEHOLDER {
                shape.1.fallback_color
            } else {
                color
            };
            assert_ne!(
                color,
                if theme == "light" {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(0x1a, 0x22, 0x20)
                },
                "heading must contrast with the modal"
            );
            assert!(
                shape
                    .0
                    .contains_rect(shape.1.galley.rect.translate(shape.1.pos.to_vec2())),
                "heading must be inside the visible clip"
            );
        }
    }
}

#[test]
fn machine_registered_cli_template_has_a_distinct_provider_label() {
    let profile = json!({
        "id":"aipro","parser":"custom","executable":"aipro","args":["run","--jsonl"],
        "input_mode":"stdin","max_sessions":3,"quota_refresh_minutes":5
    });
    let mut profiles = vec![profile.clone()];
    let snapshot = json!({"templates":[profile],"metadata_job":{},"metadata_queue":[]});
    let mut editor = Editor::default();
    editor.open(&profiles[0]);
    let mut coordinator = String::new();
    let mut roles = Default::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(900.0, 740.0),
        )),
        ..Default::default()
    };
    let mut rendered = None;
    for _ in 0..2 {
        let mut output = context.run_ui(input.clone(), |ui| {
            profile_tools::show_in_language(
                &mut profiles,
                &mut coordinator,
                &mut roles,
                &mut editor,
                profile_tools::RenderContext {
                    snapshot: &snapshot,
                    editable: true,
                    language: "en",
                    show_registry: false,
                },
                ui,
            );
        });
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        rendered = output.platform_output.accesskit_update;
    }
    assert!(
        rendered
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| { node.role() == Role::ComboBox && node.value() == Some("aipro") })
    );
}

#[test]
fn enabled_profile_save_uses_the_theme_accent_pair() {
    for theme in ["light", "dark"] {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        appearance::apply(theme, &ctx);
        let expected_bg = ctx.global_style().visuals.widgets.active.bg_fill;
        let expected_fg = ctx.global_style().visuals.widgets.active.fg_stroke.color;
        let mut profiles = Vec::new();
        let mut coordinator = String::new();
        let mut roles = Default::default();
        let mut editor = Editor::default();
        editor.open_new(
            &json!({"id":"local-agent","parser":"custom","executable":"aipro","args":[]}),
        );
        let snapshot = json!({"config":{"profiles":[]},"templates":[]});
        let mut output = None;
        for frame in 0..3 {
            let mut rendered = ctx.run_ui(
                egui::RawInput {
                    time: Some(frame as f64 * 0.5),
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1140.0, 772.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    profile_tools::show_in_language(
                        &mut profiles,
                        &mut coordinator,
                        &mut roles,
                        &mut editor,
                        profile_tools::RenderContext {
                            snapshot: &snapshot,
                            editable: true,
                            language: "en",
                            show_registry: false,
                        },
                        ui,
                    );
                },
            );
            rendered.textures_delta.set.clear();
            rendered.textures_delta.free.clear();
            output = Some(rendered);
        }
        let output = output.unwrap();
        let save = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Button && node.label() == Some("Save profile"))
            .map(|(_, node)| node.bounds().unwrap())
            .expect("enabled profile save action is accessible");
        let center = egui::pos2(
            ((save.x0 + save.x1) / 2.0) as f32,
            ((save.y0 + save.y1) / 2.0) as f32,
        );
        let has_accent_background = output.shapes.iter().any(|shape| {
            matches!(&shape.shape,
                egui::Shape::Rect(rect) if rect.fill == expected_bg && rect.rect.contains(center))
        });
        let fg = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == "Save profile" => Some(
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
            })
            .expect("save action label is rendered");
        assert!(
            has_accent_background,
            "{theme} save background must use the accent"
        );
        assert_eq!(fg, expected_fg, "{theme} save label must use accent ink");
    }
}
