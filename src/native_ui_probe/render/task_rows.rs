use super::{status_color, status_colors, status_label};
use eframe::egui::{self, Color32, RichText, Ui};

pub(super) fn show(
    ui: &mut Ui,
    status: &str,
    id: &str,
    agent: &str,
    language: &str,
    selected: Option<bool>,
) -> bool {
    let (foreground, _) = status_colors(status, ui.visuals().dark_mode);
    let waiting = matches!(status, "waiting_approval" | "waiting_tool_approval");
    let reduced_motion = ui.style().animation_time == 0.0;
    let time = ui.input(|input| input.time);
    let border = match status {
        "failed" => foreground,
        "waiting_approval" | "waiting_tool_approval" => {
            if !reduced_motion {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(250));
            }
            if reduced_motion || time.rem_euclid(1.0) < 0.5 {
                foreground
            } else {
                foreground.gamma_multiply(0.15)
            }
        }
        _ => Color32::TRANSPARENT,
    };
    let mut clicked = false;
    egui::Frame::new()
        .stroke(egui::Stroke::new(
            if waiting || status == "failed" {
                1.5
            } else {
                0.0
            },
            border,
        ))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(4, 2))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                let label = status_label(status, language);
                let accessible = crate::ui::messages::interpolate(
                    crate::ui::messages::in_language(language, "room.task_status_accessible"),
                    &[("value1", id), ("value2", label.as_ref())],
                );
                status_tag(ui, status, &label, &accessible);
                let accessible = crate::ui::messages::interpolate(
                    crate::ui::messages::in_language(language, "room.task_status_accessible"),
                    &[("value1", id), ("value2", label.as_ref())],
                );
                if let Some(selected) = selected {
                    clicked = ui
                        .add_sized(
                            egui::vec2(ui.available_width(), 26.0),
                            egui::Button::selectable(selected, format!("{id} · {agent}")),
                        )
                        .accessible_name(accessible)
                        .clicked();
                } else {
                    ui.label(format!("{id} · {agent}"));
                }
            });
        });
    clicked
}

pub(super) fn status_tag(ui: &mut Ui, status: &str, label: &str, accessible: &str) {
    let (foreground, background) = status_colors(status, ui.visuals().dark_mode);
    let time = ui.input(|input| input.time);
    if status == "running" {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(500));
    }
    egui::Frame::new()
        .fill(background)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(4.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("●")
                        .size(9.0)
                        .color(status_color(status, time)),
                );
                ui.label(RichText::new(label).small().strong().color(foreground))
                    .accessible_name(accessible);
            });
        });
}
