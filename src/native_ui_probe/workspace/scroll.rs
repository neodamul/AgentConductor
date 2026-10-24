use eframe::egui::{self, Ui};

pub(super) fn show<R>(
    ui: &mut Ui,
    body: impl FnOnce(&mut Ui) -> R,
) -> egui::scroll_area::ScrollAreaOutput<R> {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, body)
}

#[cfg(test)]
#[path = "../../../tests/native_ui_probe/workspace_scroll.rs"]
mod tests;
