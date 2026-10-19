use eframe::egui::{self, RichText};

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
pub fn show(current: usize, language: &str, ui: &mut egui::Ui) -> Option<usize> {
    ui.label(
        RichText::new(text(language, EYEBROWS[current.min(2)]))
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
    let gap = ui.spacing().item_spacing.x;
    let tab_width = ((ui.available_width() - 2.0 * gap - 1.0) / TITLES.len() as f32).max(0.0);
    ui.horizontal(|ui| {
        for (index, id) in TITLES.iter().enumerate() {
            let label = format!("{}. {}", index + 1, text(language, id));
            let response = ui
                .add_sized(
                    egui::vec2(tab_width, 48.0),
                    egui::Button::new(RichText::new(&label).size(13.0))
                        .wrap()
                        .selected(selected == index),
                )
                .accessible_name(label.clone());
            if selected == index {
                ui.painter().line_segment(
                    [response.rect.left_bottom(), response.rect.right_bottom()],
                    egui::Stroke::new(3.0, ui.visuals().hyperlink_color),
                );
            }
            if response.clicked() {
                selected = index;
            }
        }
    });
    (selected != current).then_some(selected)
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}
