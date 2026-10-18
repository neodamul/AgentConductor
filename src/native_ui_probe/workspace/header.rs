use eframe::egui::{self, CornerRadius, Frame, RichText, Stroke, vec2};

const EYEBROWS: [&str; 3] = [
    "startup.step_1_workspace",
    "startup.step_2_team",
    "startup.step_3_your_execution_decision",
];
const TITLES: [&str; 3] = [
    "startup.workspace_settings_5ebc59",
    "startup.agent_roles",
    "startup.review_the_plan",
];
const DESCRIPTIONS: [&str; 3] = [
    "startup.saves_the_workspace_name_goal_coordinator_repository_and",
    "startup.role_assignments_first_agent_preferred",
    "startup.generate_a_plan_then_review_each_role_scope",
];

pub fn show(current: usize, language: &str, ui: &mut egui::Ui) -> Option<usize> {
    ui.label(
        RichText::new(text(language, EYEBROWS[current.min(2)]))
            .small()
            .strong()
            .color(ui.visuals().hyperlink_color),
    );
    let title = text(language, TITLES[current.min(2)]);
    ui.label(
        RichText::new(title)
            .size(26.0)
            .strong()
            .color(ui.visuals().text_color()),
    )
    .accessible_name(title);
    let description = text(language, DESCRIPTIONS[current.min(2)]);
    ui.label(
        RichText::new(description)
            .small()
            .color(ui.visuals().weak_text_color()),
    )
    .accessible_name(description);
    ui.add_space(8.0);

    let mut selected = current;
    let tab_width = ((ui.available_width() - 16.0) / 3.0).max(108.0);
    ui.horizontal(|ui| {
        for (index, id) in TITLES.iter().enumerate() {
            let label = format!("{}. {}", index + 1, text(language, id));
            let active = selected == index;
            let fill = if active {
                ui.visuals().selection.bg_fill
            } else {
                ui.visuals().window_fill
            };
            let stroke = if active {
                ui.visuals().widgets.hovered.bg_stroke
            } else {
                ui.visuals().widgets.noninteractive.bg_stroke
            };
            Frame::new()
                .fill(fill)
                .stroke(Stroke::new(1.0, stroke.color))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(6.0)
                .show(ui, |ui| {
                    let color = if active {
                        ui.visuals().hyperlink_color
                    } else {
                        ui.visuals().text_color()
                    };
                    if ui
                        .add_sized(
                            vec2(tab_width, 34.0),
                            egui::Button::new(RichText::new(&label).strong().color(color))
                                .frame(false),
                        )
                        .accessible_name(label.clone())
                        .clicked()
                    {
                        selected = index;
                    }
                });
        }
    });
    (selected != current).then_some(selected)
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}
