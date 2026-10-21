#![cfg(feature = "native-ui-probe")]

use agentconductor::{manual::Block, native_ui_probe::manual_viewer};
use eframe::egui;

#[test]
fn opening_manual_does_not_push_workspace_content_down() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let blocks = [Block::Paragraph("Manual body".into())];
    let before = manual_frame(&context, false, &blocks);
    let before_y = label_y(&before, "Workspace content");
    clear(before);
    let after = manual_frame(&context, true, &blocks);
    let after_y = label_y(&after, "Workspace content");
    let visible = visible_text(&after);
    clear(after);
    assert_eq!(after_y, before_y, "manual must overlay workspace content");
    assert!(visible.iter().any(|line| line == "Manual body"));
}

#[test]
fn opening_agent_tools_guide_does_not_push_workspace_content_down() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let before = tools_frame(&context, false);
    let before_y = label_y(&before, "Workspace content");
    clear(before);
    let after = tools_frame(&context, true);
    let after_y = label_y(&after, "Workspace content");
    let visible = visible_text(&after);
    clear(after);
    assert_eq!(
        after_y, before_y,
        "tools guide must overlay workspace content"
    );
    assert!(visible.iter().any(|line| line.contains("agentmemory")));
}

fn manual_frame(context: &egui::Context, mut open: bool, blocks: &[Block]) -> egui::FullOutput {
    frame(context, |ui| {
        manual_viewer::show(&mut open, "en", blocks, ui);
    })
}

fn tools_frame(context: &egui::Context, mut open: bool) -> egui::FullOutput {
    frame(context, |ui| {
        agentconductor::native_ui_probe::workspace::navigation::tools_guide::show(
            &mut open, "en", ui,
        );
    })
}

fn frame(context: &egui::Context, mut show: impl FnMut(&mut egui::Ui)) -> egui::FullOutput {
    let mut output = None;
    for _ in 0..2 {
        let next = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000.0, 800.0),
                )),
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    show(ui);
                    ui.label("Workspace content")
                        .accessible_name("Workspace content");
                });
            },
        );
        if let Some(mut previous) = output.replace(next) {
            previous.textures_delta.set.clear();
            previous.textures_delta.free.clear();
        }
    }
    output.unwrap()
}

fn label_y(output: &egui::FullOutput, label: &str) -> f32 {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| bounds.y0 as f32)
        .unwrap_or_else(|| panic!("missing {label:?}"))
}

fn visible_text(output: &egui::FullOutput) -> Vec<String> {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect()
}

fn clear(mut output: egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
