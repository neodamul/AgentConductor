#![cfg(feature = "native-ui-probe")]

use agentconductor::{manual::Block, native_ui_probe::manual_viewer};
use eframe::egui::{self, Pos2};

#[test]
fn manual_content_table_and_screenshot_are_accessible() {
    let context = egui::Context::default();
    context.enable_accesskit();
    context.memory_mut(|memory| memory.options.screen_reader = true);
    let mut open = true;
    let blocks = [
        Block::Heading {
            level: 1,
            text: "Manual instructions".into(),
        },
        Block::Paragraph("Choose the shared folder.".into()),
        Block::Table {
            headers: vec!["Command".into(), "Purpose".into()],
            rows: vec![vec!["agentconductor --help".into(), "Show help.".into()]],
        },
        Block::Paragraph("![Accessible screen](assets/ui-workspace-current.png)".into()),
    ];
    let mut output = None;
    let mut labels = Vec::new();
    for _ in 0..2 {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(900.0, 700.0),
            )),
            ..Default::default()
        };
        let next = context.run_ui(input, |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                manual_viewer::show(&mut open, "en", &blocks, ui);
            });
        });
        if let Some(update) = next.platform_output.accesskit_update.as_ref() {
            labels.extend(update.nodes.iter().flat_map(|(_, node)| {
                [node.label(), node.value()]
                    .into_iter()
                    .flatten()
                    .map(str::to_owned)
            }));
        }
        if let Some(mut previous) = output.replace(next) {
            previous.textures_delta.set.clear();
            previous.textures_delta.free.clear();
        }
    }
    let mut output = output.unwrap();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for expected in [
        "Manual instructions",
        "Choose the shared folder.",
        "Command",
        "Purpose",
        "agentconductor --help",
        "Accessible screen",
    ] {
        assert!(
            labels.iter().any(|label| label.contains(expected)),
            "missing {expected:?}: {labels:?}"
        );
    }
}
