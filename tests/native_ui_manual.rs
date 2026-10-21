use agentconductor::{manual::Block, native_ui_probe::manual_viewer};
use eframe::egui::{self, Event, PointerButton, Pos2};

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
    let blocks = [Block::Code("agentconductor --help".into())];
    let mut output = render_manual(&context, "en", &mut open, &blocks);
    let labels = visible_text(&output);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(labels.iter().any(|label| label == "Close"));
    assert!(labels.iter().any(|label| label == "Copy"), "{labels:?}");
}

#[test]
fn native_manual_renders_bundled_screenshots() {
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
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut open = true;
        let mut output = render_manual(&context, "en", &mut open, std::slice::from_ref(&block));
        let screenshot_visible = output
            .shapes
            .iter()
            .any(|shape| matches!(&shape.shape, egui::epaint::Shape::Mesh(_)));
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        assert!(screenshot_visible, "screenshot not rendered: {markdown}");
    }
}

#[test]
fn clicking_copy_and_close_performs_both_manual_actions() {
    let code = "agentconductor --help";
    let blocks = [Block::Code(code.into())];
    for (language, copy_label, close_label) in [("en", "Copy", "Close"), ("ko", "복사", "닫기")]
    {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut open = true;
        let mut output = render_manual(&context, language, &mut open, &blocks);
        let copy = button_center(&output, |label| label.starts_with(copy_label));
        clear_textures(&mut output);
        let mut output = click_manual(&context, language, copy, &mut open, &blocks);
        assert!(
            output
                .platform_output
                .commands
                .iter()
                .any(|command| matches!(
                    command,
                    egui::OutputCommand::CopyText(text) if text == code
                )),
            "{language} Copy should put the code on the clipboard"
        );

        let close = button_center(&output, |label| label == close_label);
        clear_textures(&mut output);
        let mut output = click_manual(&context, language, close, &mut open, &blocks);
        assert!(!open, "{language} Close should dismiss the manual");
        clear_textures(&mut output);
    }
}

fn render_manual(
    context: &egui::Context,
    language: &str,
    open: &mut bool,
    blocks: &[Block],
) -> egui::FullOutput {
    let mut output = None;
    for _ in 0..2 {
        let next = context.run_ui(manual_input(Vec::new()), |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                manual_viewer::show(open, language, blocks, ui);
            });
        });
        if let Some(mut previous) = output.replace(next) {
            previous.textures_delta.set.clear();
            previous.textures_delta.free.clear();
        }
    }
    output.unwrap()
}

fn click_manual(
    context: &egui::Context,
    language: &str,
    position: Pos2,
    open: &mut bool,
    blocks: &[Block],
) -> egui::FullOutput {
    let mut output = None;
    for pressed in [true, false] {
        let event = Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        let next = context.run_ui(
            manual_input(vec![Event::PointerMoved(position), event]),
            |ui| {
                egui::CentralPanel::default()
                    .show(ui, |ui| manual_viewer::show(open, language, blocks, ui));
            },
        );
        if let Some(mut previous) = output.replace(next) {
            previous.textures_delta.set.clear();
            previous.textures_delta.free.clear();
        }
    }
    output.unwrap()
}

fn manual_input(events: Vec<Event>) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(1000.0, 700.0),
        )),
        events,
        ..Default::default()
    }
}

fn button_center(output: &egui::FullOutput, matches: impl Fn(&str) -> bool) -> Pos2 {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) if matches(&text.galley.job.text) => {
                Some(text.galley.rect.translate(text.pos.to_vec2()))
            }
            _ => None,
        })
        .map(|bounds| Pos2::new(bounds.center().x, bounds.center().y))
        .expect("button text should have screen bounds")
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

fn clear_textures(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
