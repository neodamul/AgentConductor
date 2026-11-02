#![cfg(feature = "native-ui-probe")]

use agentconductor::{
    native_ui_probe::updates::State,
    ui::update_guard::{Blocker, job_label, message},
};
use eframe::egui::{self, Pos2};

#[test]
fn update_dialog_renders_the_multiline_blocker_notice_in_english_and_korean() {
    for language in ["en", "ko"] {
        let refusal = message(
            language,
            &[
                Blocker::Active {
                    workspace: "Operations".into(),
                    what: job_label(language, "quota:claude"),
                    elapsed_ms: Some(65_000),
                },
                Blocker::Unknown {
                    what: "workspace list".into(),
                    error: "access denied".into(),
                },
            ],
        );
        let mut state = State::default();
        state.open = true;
        state.apply_status(serde_json::json!({"phase":"idle","error":refusal}));
        let context = egui::Context::default();
        context.disable_accesskit();
        let mut warmup = context.run_ui(input(), |ui| {
            state.show(language, false, ui);
        });
        warmup.textures_delta.set.clear();
        warmup.textures_delta.free.clear();
        context.enable_accesskit();
        let output = context.run_ui(input(), |ui| {
            state.show(language, false, ui);
        });
        let rendered = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let notice = rendered
            .iter()
            .find(|text| text.contains("Operations"))
            .copied()
            .unwrap_or_default();
        assert!(notice.contains("1:05"), "{notice}");
        assert!(notice.contains("access denied"), "{notice}");
        assert!(
            notice.contains('\n'),
            "blocker sections must keep their line breaks"
        );
        assert!(!notice.contains("native."), "{notice}");
    }
}

fn input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(800.0, 600.0),
        )),
        ..Default::default()
    }
}
