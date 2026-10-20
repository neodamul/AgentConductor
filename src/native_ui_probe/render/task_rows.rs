use super::{status_color_with_motion, status_colors, status_label};
use eframe::egui::{self, Color32, RichText, Ui};

pub(super) struct Agent<'a> {
    pub id: &'a str,
    pub parser: &'a str,
    pub icon_url: Option<&'a str>,
}

pub(super) fn show(
    ui: &mut Ui,
    status: &str,
    id: &str,
    agent: &Agent,
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
    let row = egui::Frame::new()
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
        .fill(if selected == Some(true) {
            ui.visuals().selection.bg_fill
        } else {
            Color32::TRANSPARENT
        })
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            // Design guide: the task name leads; a small icon, the agent name and a compact
            // status tag follow on one line that wraps instead of leaving the list.
            ui.vertical(|ui| {
                ui.label(RichText::new(id).strong());
                ui.horizontal_wrapped(|ui| {
                    let label = status_label(status, language);
                    let accessible = crate::ui::messages::interpolate(
                        crate::ui::messages::in_language(language, "room.task_status_accessible"),
                        &[("value1", id), ("value2", label.as_ref())],
                    );
                    crate::native_ui_probe::workspace::sidebar::icons::avatar_sized(
                        ui,
                        agent.id,
                        agent.parser,
                        language,
                        agent.icon_url,
                        14.0,
                    );
                    ui.label(
                        RichText::new(agent.id)
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    );
                    // A wrapped layout places the tag as one block: move it to the next line when
                    // the rest of this one is too narrow, so it never leaves the list.
                    let text_width = ui
                        .painter()
                        .layout_no_wrap(
                            label.to_string(),
                            egui::TextStyle::Small.resolve(ui.style()),
                            Color32::PLACEHOLDER,
                        )
                        .size()
                        .x;
                    if ui.available_size_before_wrap().x < text_width + 28.0 {
                        ui.end_row();
                    }
                    ui.scope(|ui| {
                        // Wrap the tag's own text when even a whole line is too narrow for it.
                        ui.set_max_width(ui.available_size_before_wrap().x.max(40.0));
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        status_tag(ui, status, &label, &accessible);
                    });
                });
            });
        });
    if let Some(selected) = selected {
        let accessible = crate::ui::messages::interpolate(
            crate::ui::messages::in_language(language, "room.task_status_accessible"),
            &[
                ("value1", id),
                ("value2", status_label(status, language).as_ref()),
            ],
        );
        let response = ui.interact(
            row.response.rect,
            ui.make_persistent_id(("session-row", id)),
            egui::Sense::click(),
        );
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::Role::Button,
                ui.is_enabled(),
                selected,
                accessible.clone(),
            )
        });
        response.clicked()
            || (response.has_focus()
                && ui.input(|input| {
                    input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)
                }))
    } else {
        false
    }
}

pub(super) fn status_tag(ui: &mut Ui, status: &str, label: &str, accessible: &str) {
    let (foreground, background) = status_colors(status, ui.visuals().dark_mode);
    let time = ui.input(|input| input.time);
    let animate = ui.style().animation_time > 0.0;
    if status == "running" && animate {
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
                        .color(status_color_with_motion(status, time, animate)),
                );
                ui.label(RichText::new(label).small().strong().color(foreground))
                    .accessible_name(accessible);
            });
        });
}
