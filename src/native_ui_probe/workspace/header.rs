use eframe::egui::{self, RichText};

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
    let eyebrow = text(
        language,
        EYEBROWS.get(current).copied().unwrap_or(EYEBROWS[0]),
    );
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
    ui.horizontal_wrapped(|ui| {
        for (index, id) in TITLES.iter().enumerate() {
            let label = format!("{}. {}", index + 1, text(language, id));
            let response = ui
                .add(
                    egui::Button::new(RichText::new(&label).size(13.0))
                        .wrap()
                        .selected(selected == index),
                )
                .accessible_name(label.clone());
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
