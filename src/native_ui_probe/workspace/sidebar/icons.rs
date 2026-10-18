use eframe::egui::{self, Color32, CornerRadius, Sense, vec2};

pub(super) fn avatar(
    ui: &mut egui::Ui,
    id: &str,
    parser: &str,
    language: &str,
    icon_url: Option<&str>,
) {
    let (rect, response) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::hover());
    if let Some(texture) = icon_url.and_then(|url| texture(ui, url)) {
        ui.painter().image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        let (fill, glyph) = match parser {
            "claude" => (Color32::from_rgb(177, 100, 76), "CC".to_owned()),
            "codex" => (Color32::from_rgb(52, 103, 91), "CX".to_owned()),
            "opencode" => (Color32::from_rgb(74, 82, 106), "OC".to_owned()),
            _ => (
                Color32::from_rgb(96, 96, 103),
                id.chars().take(2).collect::<String>().to_uppercase(),
            ),
        };
        ui.painter().rect_filled(rect, CornerRadius::same(7), fill);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            glyph,
            egui::FontId::proportional(10.0),
            Color32::WHITE,
        );
    }
    response.accessible_name(crate::ui::messages::interpolate(
        crate::ui::messages::in_language(language, "startup.agent_icon_accessible"),
        &[("value1", id)],
    ));
}

fn texture(ui: &egui::Ui, url: &str) -> Option<egui::TextureHandle> {
    let id = ui.scope_id().with(("native-provider-icon", url));
    if let Some(texture) = ui
        .ctx()
        .data(|data| data.get_temp::<egui::TextureHandle>(id))
    {
        return Some(texture);
    }
    let png = crate::icons::decode_data_url(url).ok()?;
    let image = image::load_from_memory_with_format(&png, image::ImageFormat::Png)
        .ok()?
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let image = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
    let texture =
        ui.ctx()
            .load_texture("native-provider-icon", image, egui::TextureOptions::LINEAR);
    ui.ctx()
        .data_mut(|data| data.insert_temp(id, texture.clone()));
    Some(texture)
}
