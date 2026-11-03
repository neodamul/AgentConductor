use super::super::{Event, State};
use eframe::egui::{self, Align2};
use serde_json::Value;

pub(super) fn show(
    state: &mut State,
    pending: bool,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Event> {
    if !state.scope_recovery_open {
        return None;
    }
    let text = |key| crate::ui::messages::in_language(language, key);
    let failure = state.scope_failure.clone()?;
    let preview = state.scope_preview.clone();
    let mut open = state.scope_recovery_open;
    let mut task_id = state.scope_task.clone();
    let mut paths_text = state.scope_paths.clone();
    let mut test_text = state.scope_test_command.clone();
    let mut input_changed = false;
    let mut proposal = None;
    let mut input_valid = false;
    let mut close_requested = false;
    let mut event = None;
    egui::Window::new(text("room.scope_recovery_title"))
        .id(egui::Id::NULL.with("native-scope-recovery-dialog"))
        .open(&mut open)
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(620.0)
        .default_height(380.0)
        .max_height(480.0)
        .resizable(true)
        .collapsible(false)
        .show(ui.ctx(), |ui| {
            egui::ScrollArea::vertical()
                .id_salt("scope-recovery-content")
                .max_height(380.0)
                .show(ui, |ui| {
                    ui.label(text("room.scope_recovery_description"));
                    ui.label(format!(
                        "{}{}",
                        text("room.last_error"),
                        failure["error"].as_str().unwrap_or_default()
                    ));
                    ui.separator();
                    ui.label(text("room.scope_offending_paths"));
                    egui::ScrollArea::vertical()
                        .id_salt("scope-recovery-offending")
                        .max_height(64.0)
                        .show(ui, |ui| {
                            for path in failure["offending"].as_array().into_iter().flatten() {
                                ui.monospace(path.as_str().unwrap_or_default());
                            }
                        });
                    ui.label(text("room.scope_quarantined_files"));
                    egui::ScrollArea::vertical()
                        .id_salt("scope-recovery-quarantine")
                        .max_height(72.0)
                        .show(ui, |ui| {
                            for item in failure["stray"].as_array().into_iter().flatten() {
                                ui.monospace(item["path"].as_str().unwrap_or_default());
                                ui.small(item["stored"].as_str().unwrap_or_default());
                                let key = if item["exists_at_original"] == true {
                                    "room.scope_original_exists"
                                } else {
                                    "room.scope_original_missing"
                                };
                                ui.label(text(key));
                            }
                        });

                    let tasks = failure["tasks"].as_array().cloned().unwrap_or_default();
                    let selected = tasks.iter().find(|task| task["id"] == task_id);
                    let selected_label = selected
                        .and_then(|task| task["id"].as_str())
                        .unwrap_or_else(|| text("room.unknown"));
                    let mut changed_task = false;
                    ui.horizontal(|ui| {
                        ui.label(text("room.scope_task"));
                        egui::ComboBox::from_id_salt("scope-recovery-task")
                            .selected_text(selected_label)
                            .show_ui(ui, |ui| {
                                for task in &tasks {
                                    if let Some(id) = task["id"].as_str()
                                        && ui
                                            .selectable_value(&mut task_id, id.to_owned(), id)
                                            .changed()
                                    {
                                        changed_task = true;
                                    }
                                }
                            });
                    });
                    if changed_task
                        && let Some(task) = tasks.iter().find(|item| item["id"] == task_id)
                    {
                        paths_text = strings(&task["paths"]).join("\n");
                        test_text = serde_json::to_string(&task["test_command"])
                            .unwrap_or_else(|_| "[]".into());
                    }
                    ui.label(text("room.scope_owned_paths"));
                    let paths = ui.add(
                        egui::TextEdit::multiline(&mut paths_text)
                            .desired_rows(4)
                            .desired_width(ui.available_width()),
                    );
                    let paths_changed = paths.changed();
                    paths.accessible_name(text("room.scope_owned_paths"));
                    ui.label(text("room.scope_test_command_json"));
                    let test = ui.add(
                        egui::TextEdit::singleline(&mut test_text)
                            .desired_width(ui.available_width()),
                    );
                    let test_changed = test.changed();
                    test.accessible_name(text("room.scope_test_command_json"));
                    input_changed = changed_task || paths_changed || test_changed;
                    proposal = proposal_input(&paths_text, &test_text).ok();
                    input_valid = proposal
                        .as_ref()
                        .is_some_and(|(paths, _)| !paths.is_empty());
                    if !input_valid {
                        ui.colored_label(
                            egui::Color32::RED,
                            text("room.scope_test_command_invalid"),
                        );
                    }

                    ui.separator();
                    if let Some(preview) = preview.as_ref().filter(|_| !input_changed) {
                        ui.label(text("room.scope_previous_paths"));
                        ui.label(strings(&preview["previous_paths"]).join(", "));
                        ui.label(text("room.scope_proposed_paths"));
                        ui.label(strings(&preview["paths"]).join(", "));
                        ui.label(text("room.scope_restore_results"));
                        egui::ScrollArea::vertical()
                            .id_salt("scope-recovery-restore-results")
                            .max_height(72.0)
                            .show(ui, |ui| {
                                for item in preview["restore"].as_array().into_iter().flatten() {
                                    ui.horizontal(|ui| {
                                        ui.monospace(item["path"].as_str().unwrap_or_default());
                                        ui.label(item["action"].as_str().unwrap_or_default());
                                    });
                                }
                            });
                        ui.label(text("room.scope_recovery_resume_manually"));
                        ui.label(format!(
                            "{}{}",
                            text("room.scope_digest"),
                            preview["digest"].as_str().unwrap_or_default()
                        ));
                    }
                });
            if preview.is_some() && !input_changed {
                ui.small(text("room.scope_recovery_resume_manually"));
            }
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        !pending && !input_changed && !task_id.is_empty() && input_valid,
                        egui::Button::new(text("room.scope_preview")),
                    )
                    .clicked()
                    && let Some((paths, test)) = proposal.clone()
                {
                    event = Some(Event::ScopeRecoveryPreview {
                        task: task_id.clone(),
                        paths,
                        test,
                    });
                }
                let approved = preview
                    .as_ref()
                    .filter(|value| !input_changed && value["task"] == task_id);
                if let Some(value) = approved
                    && ui
                        .add_enabled(
                            !pending,
                            egui::Button::new(text("room.scope_approve_recovery")),
                        )
                        .clicked()
                    && let Some((paths, test)) = proposal
                    && let Some(digest) = value["digest"].as_str()
                {
                    event = Some(Event::ScopeRecoveryApply {
                        task: task_id.clone(),
                        paths,
                        test,
                        digest: digest.into(),
                    });
                }
                if ui
                    .add_enabled(!pending, egui::Button::new(text("room.close")))
                    .clicked()
                {
                    close_requested = true;
                }
            });
        });
    if close_requested {
        open = false;
    }
    state.scope_recovery_open = open;
    state.scope_task = task_id;
    state.scope_paths = paths_text;
    state.scope_test_command = test_text;
    if input_changed {
        state.scope_preview = None;
    }
    event
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn proposal_input(paths: &str, test: &str) -> Result<(Vec<String>, Option<Vec<String>>), String> {
    let paths: Vec<String> = paths
        .lines()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect();
    let test: Vec<String> =
        serde_json::from_str(test).map_err(|_| "test command must be a JSON argv array")?;
    Ok((paths, Some(test)))
}
