use super::State;
use eframe::egui::{self, Pos2, Rect, Stroke, vec2};

impl State {
    pub fn show_launcher(&mut self, language: &str, ui: &mut egui::Ui) {
        let label = crate::ui::messages::in_language(language, "chat.open");
        let response = ui
            .add_sized(vec2(34.0, 30.0), egui::Button::new(""))
            .on_hover_text(label)
            .accessible_name(label);
        paint_icon(ui, response.rect);
        if response.clicked() {
            self.open = true;
        }
    }
}

fn paint_icon(ui: &egui::Ui, button: Rect) {
    let visuals = ui.visuals();
    let ink = visuals.text_color();
    let bubble = Rect::from_center_size(button.center() + vec2(0.0, 1.0), vec2(20.0, 15.0));
    let stroke = Stroke::new(1.5, ink);
    ui.painter()
        .rect_stroke(bubble, 4.0, stroke, egui::StrokeKind::Middle);
    ui.painter().line_segment(
        [
            Pos2::new(bubble.left() + 5.0, bubble.bottom() - 0.5),
            Pos2::new(bubble.left() + 3.0, bubble.bottom() + 3.0),
        ],
        stroke,
    );
    ui.painter().line_segment(
        [
            Pos2::new(bubble.left() + 3.0, bubble.bottom() + 3.0),
            Pos2::new(bubble.left() + 8.0, bubble.bottom() - 0.5),
        ],
        stroke,
    );
    ui.painter().line_segment(
        [
            Pos2::new(bubble.center().x, bubble.top() - 0.5),
            Pos2::new(bubble.center().x, bubble.top() - 4.0),
        ],
        stroke,
    );
    ui.painter()
        .circle_filled(Pos2::new(bubble.center().x, bubble.top() - 5.0), 1.5, ink);
    for x in [-4.0, 4.0] {
        ui.painter().circle_filled(
            Pos2::new(bubble.center().x + x, bubble.center().y),
            1.25,
            ink,
        );
    }
}
