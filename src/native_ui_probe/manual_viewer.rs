use crate::manual::Block;
use eframe::egui;
use std::collections::HashMap;

#[derive(Clone, Default)]
struct ImageCache(HashMap<String, egui::TextureHandle>);

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
    let was_open = *open;
    let mut close_requested = false;
    egui::Window::new(text("startup.user_manual"))
        .id(egui::Id::NULL.with("native-user-manual"))
        .open(open)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .default_open(true)
        .resizable(true)
        .default_width(720.0)
        .max_height(560.0)
        .vscroll(true)
        .show(ui.ctx(), |ui| {
            if ui.button(text("startup.close")).clicked() {
                close_requested = true;
            }
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
                        if let Some((alt, source)) = image_reference(text) {
                            if let Some(texture) = screenshot(ui.ctx(), source) {
                                ui.add(
                                    egui::Image::new(&texture)
                                        .max_width(ui.available_width())
                                        .max_height(440.0)
                                        .alt_text(alt),
                                );
                                ui.small(alt);
                            } else {
                                ui.label(alt);
                            }
                        } else {
                            ui.label(inline_text(text));
                        }
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
                            egui::Grid::new(("manual-table", index))
                                .striped(true)
                                .show(ui, |ui| {
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
                                });
                        });
                    }
                }
            }
        });
    if close_requested {
        *open = false;
    }
    if was_open && !*open {
        clear_screenshots(ui.ctx());
    }
}

fn cache_id() -> egui::Id {
    egui::Id::unique("native-manual-screenshots")
}

fn clear_screenshots(context: &egui::Context) {
    context.data_mut(|data| data.insert_temp(cache_id(), ImageCache::default()));
}

fn image_reference(markdown: &str) -> Option<(&str, &str)> {
    let (alt, source) = markdown.strip_prefix("![")?.split_once("](")?;
    Some((alt, source.strip_suffix(')')?))
}

fn screenshot(context: &egui::Context, source: &str) -> Option<egui::TextureHandle> {
    if let Some(texture) = context.data(|data| {
        data.get_temp::<ImageCache>(cache_id())
            .and_then(|cache| cache.0.get(source).cloned())
    }) {
        return Some(texture);
    }
    let bytes: &[u8] = match source {
        "assets/ui-workspace-current.png" => {
            include_bytes!("../../docs/assets/ui-workspace-current.png")
        }
        "assets/ui-workspace-korean-current.png" => {
            include_bytes!("../../docs/assets/ui-workspace-korean-current.png")
        }
        "assets/ui-workspace-dark-current.png" => {
            include_bytes!("../../docs/assets/ui-workspace-dark-current.png")
        }
        "assets/ui-agent-settings-current.png" => {
            include_bytes!("../../docs/assets/ui-agent-settings-current.png")
        }
        "assets/ui-agent-room-native.jpg" => {
            include_bytes!("../../docs/assets/ui-agent-room-native.jpg")
        }
        _ => return None,
    };
    let decoded = image::load_from_memory(bytes).ok()?.to_rgba8();
    let size = [decoded.width() as usize, decoded.height() as usize];
    let image = egui::ColorImage::from_rgba_unmultiplied(size, decoded.as_raw());
    let texture = context.load_texture(source, image, egui::TextureOptions::LINEAR);
    context.data_mut(|data| {
        data.get_temp_mut_or_default::<ImageCache>(cache_id())
            .0
            .insert(source.to_owned(), texture.clone());
    });
    Some(texture)
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
