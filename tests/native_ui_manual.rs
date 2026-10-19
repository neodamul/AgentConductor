use agentconductor::{manual::Block, native_ui_probe::manual_viewer};
use eframe::egui::{self, accesskit::Role};

#[test]
fn bundled_manuals_keep_command_tables_and_code_as_native_blocks() {
    for language in ["en", "ko"] {
        let blocks = manual_viewer::blocks(language);
        assert!(
            blocks
                .iter()
                .any(|block| matches!(block, Block::Table { .. }))
        );
        assert!(blocks.iter().any(|block| matches!(block, Block::Code(_))));
    }
    assert!(
        matches!(manual_viewer::blocks("ko").first(), Some(Block::Heading { text, .. }) if text.contains("사용 설명서"))
    );
}

#[test]
fn native_manual_exposes_close_and_code_copy_controls() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut open = true;
    let blocks = manual_viewer::blocks("en");
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1200.0, 1200.0),
        )),
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| {
        manual_viewer::show(&mut open, "en", &blocks, ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(labels.contains(&"Close"));
    assert!(labels.iter().any(|label| label.starts_with("Copy ")));
}

#[test]
fn native_manual_renders_bundled_screenshots_with_accessible_alt_text() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1000.0, 700.0),
        )),
        ..Default::default()
    };
    let screenshots = ["en", "ko"]
        .into_iter()
        .flat_map(manual_viewer::blocks)
        .filter_map(|block| {
            let markdown = match &block {
                Block::Paragraph(text) if text.starts_with("![") => text.clone(),
                _ => return None,
            };
            Some((block, markdown))
        })
        .collect::<Vec<_>>();
    assert!(screenshots.len() >= 4);
    for (block, markdown) in screenshots {
        let alt = markdown
            .strip_prefix("![")
            .unwrap()
            .split_once("](")
            .unwrap()
            .0;
        let mut open = true;
        let mut output = context.run_ui(input.clone(), |ui| {
            manual_viewer::show(&mut open, "en", std::slice::from_ref(&block), ui);
        });
        let screenshot_visible = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Image && node.label() == Some(alt));
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        assert!(screenshot_visible, "screenshot not rendered: {markdown}");
    }
}
