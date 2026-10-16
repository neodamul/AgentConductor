use crate::manual::Block;
use eframe::egui;

pub fn blocks(language: &str) -> Vec<Block> {
    let source = if language == "ko" {
        include_str!("../../docs/user-manual.ko.md")
    } else {
        include_str!("../../docs/user-manual.md")
    };
    crate::manual::blocks(source)
}

pub fn show(open: &mut bool, language: &str, blocks: &[Block], ui: &mut egui::Ui) {
    if !*open {
        return;
    }
    let text = |id| crate::ui::messages::in_language(language, id);
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.heading(text("startup.user_manual"));
            if ui.button(text("startup.close")).clicked() {
                *open = false;
            }
        });
        egui::ScrollArea::vertical()
            .max_height(620.0)
            .show(ui, |ui| {
                for (index, block) in blocks.iter().enumerate() {
                    match block {
                        Block::Heading { level, text: title } => {
                            let title = inline_text(title);
                            if *level <= 2 {
                                ui.heading(title);
                            } else {
                                ui.strong(title);
                            }
                        }
                        Block::Paragraph(text) => {
                            ui.label(inline_text(text));
                        }
                        Block::List(items) => {
                            for item in items {
                                ui.label(format!("• {}", inline_text(item)));
                            }
                        }
                        Block::Code(code) => {
                            ui.group(|ui| {
                                let summary = code
                                    .lines()
                                    .find(|line| !line.trim().is_empty())
                                    .unwrap_or_default();
                                if ui
                                    .button(text("native.copy"))
                                    .accessible_name(format!("{} {summary}", text("native.copy")))
                                    .clicked()
                                {
                                    ui.ctx().copy_text(code.clone());
                                }
                                egui::ScrollArea::horizontal().show(ui, |ui| ui.code(code));
                            });
                        }
                        Block::Table { headers, rows } => {
                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                egui::Grid::new(("manual-table", index)).striped(true).show(
                                    ui,
                                    |ui| {
                                        for header in headers {
                                            ui.strong(inline_text(header));
                                        }
                                        ui.end_row();
                                        for row in rows {
                                            for cell in row {
                                                ui.label(inline_text(cell));
                                            }
                                            ui.end_row();
                                        }
                                    },
                                );
                            });
                        }
                    }
                }
            });
    });
}

fn inline_text(markdown: &str) -> String {
    let mut out = String::new();
    let mut rest = markdown;
    while !rest.is_empty() {
        let marker = if rest.starts_with("![") {
            Some((2, true))
        } else if rest.starts_with('[') {
            Some((1, false))
        } else {
            None
        };
        if let Some((start, image)) = marker
            && let Some(label_end) = rest[start..].find("](")
            && let Some(url_end) = rest[start + label_end + 2..].find(')')
        {
            if image {
                out.push_str("Screenshot: ");
            }
            out.push_str(&rest[start..start + label_end]);
            rest = &rest[start + label_end + 2 + url_end + 1..];
        } else if let Some(strong) = rest.strip_prefix("**")
            && let Some(end) = strong.find("**")
        {
            out.push_str(&strong[..end]);
            rest = &strong[end + 2..];
        } else if let Some(code) = rest.strip_prefix('`')
            && let Some(end) = code.find('`')
        {
            out.push_str(&code[..end]);
            rest = &code[end + 1..];
        } else {
            let ch = rest.chars().next().expect("nonempty");
            out.push(ch);
            rest = &rest[ch.len_utf8()..];
        }
    }
    out
}
