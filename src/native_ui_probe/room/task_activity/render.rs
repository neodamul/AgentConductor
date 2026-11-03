use super::*;

pub(super) fn entry(ui: &mut egui::Ui, item: Entry) {
    let color = match item.tone {
        Tone::Normal => ui.visuals().text_color(),
        Tone::Muted => ui.visuals().weak_text_color(),
        Tone::Tool => ui.visuals().hyperlink_color,
        Tone::Result => ui.visuals().text_color(),
        Tone::Warning => ui.visuals().warn_fg_color,
        Tone::Error => ui.visuals().error_fg_color,
    };
    let width = ui.available_width();
    ui.group(|ui| {
        ui.set_min_width(width);
        ui.label(RichText::new(&item.label).strong().color(color))
            .accessible_name(item.label);
        ui.label(&item.text).accessible_name(item.text);
    });
}

pub(super) fn localized(key: &str, language: &str) -> String {
    let key = match key {
        "room.session" => crate::ui::messages::in_language(language, "room.session"),
        "room.agent" => crate::ui::messages::in_language(language, "room.agent"),
        "room.tool" => crate::ui::messages::in_language(language, "room.tool"),
        "room.thinking" => crate::ui::messages::in_language(language, "room.thinking"),
        "room.tool_error" => crate::ui::messages::in_language(language, "room.tool_error"),
        "result" => crate::ui::messages::in_language(language, "room.result"),
        "room.result" => crate::ui::messages::in_language(language, "room.result"),
        "room.failed_031a8f" => crate::ui::messages::in_language(language, "room.failed_031a8f"),
        "room.finished" => crate::ui::messages::in_language(language, "room.finished"),
        "room.error" => crate::ui::messages::in_language(language, "room.error"),
        "room.warning" => crate::ui::messages::in_language(language, "room.warning"),
        "room.usage" => crate::ui::messages::in_language(language, "room.usage"),
        "room.plan" => crate::ui::messages::in_language(language, "room.plan"),
        "room.status" => crate::ui::messages::in_language(language, "room.status"),
        other => other,
    };
    key.into()
}

pub(super) fn clip(value: &str, maximum: usize) -> String {
    let mut chars = value.chars();
    let output = chars.by_ref().take(maximum).collect::<String>();
    if chars.next().is_some() {
        format!("{output}…")
    } else {
        output
    }
}
