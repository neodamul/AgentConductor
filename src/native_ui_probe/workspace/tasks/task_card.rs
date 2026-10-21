use super::{agent_picker, fields, text_field};
use eframe::egui::{self, Ui};
use serde_json::Value;

pub(super) fn show(
    ui: &mut Ui,
    index: usize,
    task: &mut Value,
    can_remove: bool,
    agents: &[Value],
    language: &str,
) -> bool {
    let text = |id| crate::ui::messages::in_language(language, id);
    let id = task["id"]
        .as_str()
        .unwrap_or(text("startup.task"))
        .to_owned();
    let summary = instruction_summary(task["prompt"].as_str().unwrap_or_default());
    let heading = format!("{:02} · {id}", index + 1);
    let mut remove = false;
    ui.push_id(("review-task", id.clone()), |ui| {
        ui.horizontal(|ui| {
            let response = ui.button("›");
            let popup_id = egui::Popup::default_response_id(&response);
            let label = text(if egui::Popup::is_id_open(ui.ctx(), popup_id) {
                "startup.collapse_task"
            } else {
                "startup.expand_task"
            });
            let response = response.accessible_name(crate::ui::messages::interpolate(
                label,
                &[("value1", id.as_str())],
            ));
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new(&heading).truncate())
                        .accessible_name(heading.clone());
                    if let Some(role) = task["role"].as_str() {
                        let label = role_label(language, role);
                        ui.label(
                            egui::RichText::new(label)
                                .small()
                                .color(ui.visuals().hyperlink_color),
                        )
                        .accessible_name(label);
                    }
                });
                ui.add(egui::Label::new(egui::RichText::new(&summary).small()).truncate())
                    .accessible_name(summary.clone());
            });
            if can_remove && ui.small_button(text("startup.remove_task")).clicked() {
                remove = true;
            }
            egui::Popup::menu(&response)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .show(|ui| {
                    ui.set_max_width(560.0);
                    egui::ScrollArea::vertical()
                        .max_height(560.0)
                        .show(ui, |ui| {
                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.indent(("review-task-body", id), |ui| {
                                    text_field(ui, language, "startup.task_id", task, "id");
                                    agent_picker(ui, language, task, agents);
                                    fields::model_picker(ui, language, task, agents);
                                    fields::execution_option_pickers(ui, language, task, agents);
                                    text_field(
                                        ui,
                                        language,
                                        "startup.instructions",
                                        task,
                                        "prompt",
                                    );
                                    text_field(
                                        ui,
                                        language,
                                        "startup.owned_paths_one_per_line_label",
                                        task,
                                        "paths",
                                    );
                                    text_field(
                                        ui,
                                        language,
                                        "startup.dependencies_comma_separated_label",
                                        task,
                                        "depends_on",
                                    );
                                    text_field(
                                        ui,
                                        language,
                                        "startup.test_command_json_argv",
                                        task,
                                        "test_command",
                                    );
                                });
                            });
                        });
                });
        });
    });
    remove
}

fn role_label<'a>(language: &str, role: &'a str) -> &'a str {
    let key = match role {
        "verification" => "startup.verification",
        "development" => "startup.development",
        "documentation" => "startup.documentation",
        "knowledge" => "startup.knowledge_assetization",
        "testing" => "startup.testing",
        _ => return role,
    };
    crate::ui::messages::in_language(language, key)
}

fn instruction_summary(prompt: &str) -> String {
    prompt
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.ends_with(':'))
        .map(|line| line.trim_start_matches(['-', '*', ' ']))
        .map(|line| {
            let mut chars = line.chars();
            let mut summary: String = chars.by_ref().take(160).collect();
            if chars.next().is_some() {
                summary.push('…');
            }
            summary
        })
        .unwrap_or_default()
}
