#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::sidebar;
use eframe::egui;
use serde_json::json;

#[test]
fn sidebar_omits_static_local_first_tagline_in_both_languages() {
    for (language, labels) in [
        (
            "en",
            [
                "LOCAL BY DEFAULT",
                "One application. Independent agent sessions. Your approval controls execution and final merge.",
            ],
        ),
        (
            "ko",
            [
                "기본은 로컬",
                "하나의 앱, 독립된 에이전트 세션. 실행과 최종 병합은 당신의 승인으로 결정됩니다.",
            ],
        ),
    ] {
        let context = egui::Context::default();
        context.enable_accesskit();
        let run = || {
            context.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(248.0, 800.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    egui::Panel::left("sidebar-test")
                        .default_size(248.0)
                        .show(ui, |ui| {
                            sidebar::show(&[], &json!({}), language, false, false, ui);
                        });
                },
            )
        };
        let mut first = run();
        first.textures_delta.set.clear();
        first.textures_delta.free.clear();
        let mut output = run();
        let rendered_text = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ");
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        for label in labels {
            assert!(
                !rendered_text.contains(label),
                "{language}: {rendered_text}"
            );
        }
    }
}
