use super::{Action, State};
use eframe::egui::{self, RichText};
use serde_json::Value;

pub fn show(
    state: &mut State,
    snapshot: &Value,
    pending: bool,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Action> {
    let text = |id| crate::ui::messages::in_language(language, id);
    let plan = &snapshot["report"]["plan"];
    let tasks = plan["tasks"].as_array().filter(|tasks| !tasks.is_empty())?;
    if !tasks.iter().any(|task| task["id"] == state.selected_task) {
        state.selected_task = tasks[0]["id"].as_str().unwrap_or_default().into();
    }
    ui.separator();
    ui.heading(text("room.task_inspector"))
        .accessible_name(text("room.task_inspector"));
    let previous = state.selected_task.clone();
    egui::ComboBox::from_id_salt("native-task-picker")
        .selected_text(&state.selected_task)
        .show_ui(ui, |ui| {
            for task in tasks {
                if let Some(id) = task["id"].as_str() {
                    ui.selectable_value(&mut state.selected_task, id.to_owned(), id);
                }
            }
        });
    let plan_task = tasks
        .iter()
        .find(|task| task["id"] == state.selected_task)?;
    let task_report = snapshot["report"]["tasks"].as_array().and_then(|reports| {
        reports
            .iter()
            .find(|report| report["id"] == state.selected_task)
    });
    let current_agent = plan_task["agent"].as_str().unwrap_or_default();
    let current_model = plan_task["model"].as_str().unwrap_or_default();
    if previous != state.selected_task
        || state.assigned_agent != current_agent
        || state.target_agent.is_empty()
    {
        state.target_agent = current_agent.into();
        state.assigned_agent = current_agent.into();
    }
    if state.model_task != state.selected_task
        || state.model_agent != current_agent
        || state.assigned_model != current_model
    {
        state.model_task.clone_from(&state.selected_task);
        state.model_agent = current_agent.into();
        state.assigned_model = current_model.into();
        state.selected_model = current_model.into();
    }
    if state.feedback_task.is_empty() {
        state.feedback_task = state.selected_task.clone();
    }
    if previous != state.selected_task {
        if !state.feedback_task.is_empty() {
            state.feedback_by_task.insert(
                state.feedback_task.clone(),
                std::mem::take(&mut state.feedback),
            );
        }
        state.feedback_task = state.selected_task.clone();
        state.feedback = state
            .feedback_by_task
            .remove(&state.selected_task)
            .unwrap_or_default();
    }
    if state.schedule_task != state.selected_task {
        if !state.schedule_task.is_empty() {
            state.schedule_by_task.insert(
                state.schedule_task.clone(),
                std::mem::take(&mut state.schedule_at),
            );
        }
        state.schedule_task.clone_from(&state.selected_task);
        state.schedule_at = state
            .schedule_by_task
            .remove(&state.selected_task)
            .or_else(|| {
                task_report
                    .and_then(|report| report["restriction"]["available_at"].as_i64())
                    .and_then(schedule_timestamp)
            })
            .unwrap_or_default();
    }
    let saved_feedback = task_report.and_then(|report| report["feedback"].as_str());
    if saved_feedback == Some(state.feedback.as_str()) {
        state.feedback.clear();
    }
    if task_report.is_some_and(|report| {
        report["restriction"]["source"] == "manual_schedule"
            && task_report.and_then(|task| task["restriction"]["available_at"].as_i64())
                == crate::availability::parse_time(&state.schedule_at).ok()
    }) {
        state.schedule_at.clear();
    }
    let agents = plan["agents"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    let target_is_registered = agents
        .iter()
        .any(|agent| agent["id"].as_str() == Some(state.target_agent.as_str()));
    let models = agents
        .iter()
        .find(|agent| agent["id"] == current_agent)
        .and_then(|agent| agent["catalog"]["models"].as_array())
        .map(Vec::as_slice)
        .unwrap_or_default();
    let editable = !pending
        && snapshot["job"]["state"] != "running"
        && !matches!(
            snapshot["report"]["state"].as_str(),
            Some("running" | "completed")
        )
        && !snapshot["report"]["tasks"]
            .as_array()
            .is_some_and(|tasks| tasks.iter().any(|task| task["status"] == "running"));
    let mut event = None;
    ui.horizontal(|ui| {
        ui.label(text("room.agent"));
        egui::ComboBox::from_id_salt("native-task-agent")
            .selected_text(&state.target_agent)
            .show_ui(ui, |ui| {
                for agent in agents {
                    if let Some(id) = agent["id"].as_str() {
                        ui.selectable_value(&mut state.target_agent, id.to_owned(), id);
                    }
                }
            });
        if ui
            .add_enabled(
                editable && target_is_registered && state.target_agent != current_agent,
                egui::Button::new(text("room.assign")),
            )
            .accessible_name(text("room.assign"))
            .clicked()
        {
            event = Some(Action::Reassign {
                task: state.selected_task.clone(),
                agent: state.target_agent.clone(),
            });
        }
    });
    let session_id = task_report.and_then(|report| report["session_id"].as_str());
    let session_text = format!(
        "{}{}",
        text("room.session_168693"),
        session_id.unwrap_or(text("room.no_native_session_yet"))
    );
    ui.add(egui::Label::new(&session_text).wrap())
        .accessible_name(session_text.clone());
    let model_label = ui.label(text("startup.session_model"));
    let selected_model_label = if state.selected_model.is_empty() {
        text("startup.cli_default").to_owned()
    } else {
        state.selected_model.clone()
    };
    ui.vertical(|ui| {
        let selection = ui.add_enabled_ui(editable && !models.is_empty(), |ui| {
            egui::ComboBox::from_id_salt("native-task-model")
                .selected_text(selected_model_label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut state.selected_model,
                        String::new(),
                        text("startup.cli_default"),
                    );
                    for model in models {
                        if let Some(model) = model.as_str() {
                            ui.selectable_value(&mut state.selected_model, model.to_owned(), model);
                        }
                    }
                })
        });
        selection
            .response
            .labelled_by(model_label.id)
            .accessible_name(text("startup.session_model"));
        if models.is_empty() {
            ui.small(text("room.no_observed_catalog"));
        }
        if ui
            .add_enabled(
                editable
                    && !state.selected_model.is_empty()
                    && state.selected_model != current_model
                    && models
                        .iter()
                        .any(|model| model.as_str() == Some(state.selected_model.as_str())),
                egui::Button::new(text("room.set")),
            )
            .accessible_name(text("room.set"))
            .clicked()
        {
            event = Some(Action::SetModel {
                task: state.selected_task.clone(),
                model: state.selected_model.clone(),
            });
        }
    });
    if task_report.is_some_and(|report| report["status"] == "waiting_availability") {
        ui.separator();
        ui.heading(text("room.availability"))
            .accessible_name(text("room.availability"));
        if let Some(reason) =
            task_report.and_then(|report| report["restriction"]["reason"].as_str())
        {
            ui.label(reason.chars().take(500).collect::<String>());
        }
        let schedule_label = ui.label(text("room.schedule_timestamp_with_timezone"));
        ui.add_enabled(
            editable,
            egui::TextEdit::singleline(&mut state.schedule_at)
                .hint_text(text("room.enter_explicit_schedule_time")),
        )
        .labelled_by(schedule_label.id)
        .accessible_name(text("room.schedule_timestamp_with_timezone"));
        ui.small(text("room.enter_explicit_schedule_time"));
        if ui
            .add_enabled(
                editable && crate::availability::parse_time(&state.schedule_at).is_ok(),
                egui::Button::new(text("room.schedule_retry")),
            )
            .accessible_name(text("room.schedule_retry"))
            .clicked()
        {
            event = Some(Action::Schedule {
                task: state.selected_task.clone(),
                at: state.schedule_at.clone(),
            });
        }
    }
    let label = ui.label(text("room.your_direction"));
    ui.add_enabled(
        editable,
        egui::TextEdit::multiline(&mut state.feedback)
            .desired_rows(3)
            .desired_width(ui.available_width()),
    )
    .labelled_by(label.id)
    .accessible_name(text("room.your_direction"));
    ui.add(
        egui::Label::new(
            RichText::new(text("room.targeted_revision_previous_evidence_is_retained"))
                .color(ui.visuals().weak_text_color()),
        )
        .wrap(),
    );
    if ui
        .add_enabled(
            editable && !state.feedback.trim().is_empty() && state.feedback.len() <= 16 * 1024,
            egui::Button::new(text("room.send_feedback_823598")),
        )
        .accessible_name(text("room.send_feedback_823598"))
        .clicked()
    {
        event = Some(Action::Feedback {
            task: state.selected_task.clone(),
            feedback: state.feedback.clone(),
        });
    }
    event
}

fn schedule_timestamp(timestamp: i64) -> Option<String> {
    let value = time::OffsetDateTime::from_unix_timestamp(timestamp).ok()?;
    let year = value.year();
    (0..=9999).contains(&year).then(|| {
        format!(
            "{year:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            u8::from(value.month()),
            value.day(),
            value.hour(),
            value.minute(),
            value.second()
        )
    })
}
