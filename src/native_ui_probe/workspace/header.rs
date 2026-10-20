use eframe::egui::{self, Align2, Color32, FontId, Pos2, RichText, Stroke, vec2};

const TITLES: [&str; 3] = [
    "startup.workspace_settings_5ebc59",
    "startup.agent_roles",
    "startup.review_the_plan",
];
const EYEBROWS: [&str; 3] = [
    "startup.step_1_workspace",
    "startup.step_2_team",
    "startup.step_3_your_execution_decision",
];
pub fn show(current: usize, language: &str, ui: &mut egui::Ui) -> Option<usize> {
    let eyebrow = step_eyebrow(current, language);
    ui.label(
        RichText::new(eyebrow)
            .small()
            .strong()
            .color(ui.visuals().hyperlink_color),
    );
    let title = text(language, "startup.what_should_the_team_work_on");
    ui.heading(
        RichText::new(title)
            .size(26.0)
            .strong()
            .color(ui.visuals().text_color()),
    )
    .accessible_name(title);
    ui.add_space(12.0);

    let mut selected = current;
    let step_width =
        ((ui.available_width() - ui.spacing().item_spacing.x * 2.0) / TITLES.len() as f32).floor();
    ui.horizontal(|ui| {
        for (index, id) in TITLES.iter().enumerate() {
            let label = format!("{}. {}", index + 1, text(language, id));
            let active = selected == index;
            let visuals = ui.visuals();
            let accent = visuals.selection.stroke.color;
            let dark_mode = visuals.dark_mode;
            let inactive_fill = visuals.widgets.inactive.bg_fill;
            let text_color = visuals.text_color();
            let panel_fill = visuals.panel_fill;
            let badge_fill = if active { accent } else { inactive_fill };
            let badge_ink = if active && !dark_mode {
                Color32::WHITE
            } else {
                text_color
            };
            let background = if active {
                visuals.selection.bg_fill
            } else {
                panel_fill
            };
            let response = ui.add_sized(
                vec2(step_width, 50.0),
                egui::Button::new("").fill(background).stroke(Stroke::NONE),
            );
            let rect = response.rect;
            ui.painter().line_segment(
                [rect.left_bottom(), rect.right_bottom()],
                Stroke::new(if active { 3.0 } else { 1.0 }, accent),
            );
            let badge_center = Pos2::new(rect.left() + 24.0, rect.center().y);
            ui.painter().circle_filled(badge_center, 11.0, badge_fill);
            ui.painter().text(
                badge_center,
                Align2::CENTER_CENTER,
                (index + 1).to_string(),
                FontId::proportional(12.0),
                badge_ink,
            );
            ui.painter().text(
                Pos2::new(rect.left() + 42.0, rect.center().y),
                Align2::LEFT_CENTER,
                text(language, id),
                FontId::proportional(13.0),
                text_color,
            );
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::Role::Button,
                    ui.is_enabled(),
                    active,
                    label.clone(),
                )
            });
            if response.clicked()
                || (response.has_focus()
                    && ui.input(|input| {
                        input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)
                    }))
            {
                selected = index;
            }
        }
    });
    (selected != current).then_some(selected)
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

pub fn step_eyebrow(step: usize, language: &str) -> &'static str {
    text(language, EYEBROWS.get(step).copied().unwrap_or(EYEBROWS[0]))
}

pub fn show_card_eyebrow(step: usize, language: &str, ui: &mut egui::Ui) {
    ui.label(
        RichText::new(step_eyebrow(step, language))
            .small()
            .strong()
            .color(ui.visuals().hyperlink_color),
    );
}
