use eframe::egui::{self, RichText};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Feedback { task: String, feedback: String },
    Reassign { task: String, agent: String },
    SetModel { task: String, model: String },
}

#[derive(Default)]
pub struct State {
    selected_task: String,
    target_agent: String,
    assigned_agent: String,
    selected_model: String,
    model_task: String,
    model_agent: String,
    assigned_model: String,
    feedback: String,
    feedback_task: String,
    feedback_by_task: HashMap<String, String>,
}

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
    let saved_feedback = snapshot["report"]["tasks"]
        .as_array()
        .and_then(|reports| {
            reports
                .iter()
                .find(|report| report["id"] == state.selected_task)
        })
        .and_then(|report| report["feedback"].as_str());
    if saved_feedback == Some(state.feedback.as_str()) {
        state.feedback.clear();
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
    let session_id = snapshot["report"]["tasks"]
        .as_array()
        .and_then(|reports| {
            reports
                .iter()
                .find(|report| report["id"] == state.selected_task)
        })
        .and_then(|report| report["session_id"].as_str());
    let session_text = format!(
        "{}{}",
        text("room.session_168693"),
        session_id.unwrap_or(text("room.no_native_session_yet"))
    );
    ui.label(&session_text)
        .accessible_name(session_text.clone());
    let model_label = ui.label(text("startup.session_model"));
    let selected_model_label = if state.selected_model.is_empty() {
        text("startup.cli_default").to_owned()
    } else {
        state.selected_model.clone()
    };
    ui.horizontal(|ui| {
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
            ui.label(text("room.no_observed_catalog"));
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
    let label = ui.label(text("room.your_direction"));
    ui.add_enabled(
        editable,
        egui::TextEdit::multiline(&mut state.feedback).desired_rows(3),
    )
    .labelled_by(label.id)
    .accessible_name(text("room.your_direction"));
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(text("room.targeted_revision_previous_evidence_is_retained"))
                .color(ui.visuals().weak_text_color()),
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
    });
    event
}

pub fn request(snapshot: &Value, action: Action) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .filter(|revision| !revision.is_empty())
        .ok_or("current run revision is unavailable")?;
    let report = &snapshot["report"];
    if snapshot["job"]["state"] == "running"
        || matches!(report["state"].as_str(), Some("running" | "completed"))
        || report["tasks"]
            .as_array()
            .is_some_and(|tasks| tasks.iter().any(|task| task["status"] == "running"))
    {
        return Err("stop the active run before changing task settings".into());
    }
    let task_id = match &action {
        Action::Feedback { task, .. }
        | Action::Reassign { task, .. }
        | Action::SetModel { task, .. } => task,
    };
    let exists = report["plan"]["tasks"].as_array().is_some_and(|tasks| {
        tasks
            .iter()
            .any(|task| task["id"].as_str() == Some(task_id.as_str()))
    }) && report["tasks"].as_array().is_some_and(|tasks| {
        tasks
            .iter()
            .any(|task| task["id"].as_str() == Some(task_id.as_str()))
    });
    if !exists {
        return Err("unknown task/session ID".into());
    }
    let action = match action {
        Action::Feedback { task, feedback }
            if !feedback.trim().is_empty() && feedback.len() <= 16 * 1024 =>
        {
            serde_json::json!({"kind":"revise","task":task,"feedback":feedback})
        }
        Action::Feedback { .. } => {
            return Err("review feedback must contain 1..16384 bytes".into());
        }
        Action::Reassign { task, agent }
            if report["plan"]["agents"]
                .as_array()
                .is_some_and(|agents| agents.iter().any(|profile| profile["id"] == agent)) =>
        {
            serde_json::json!({"kind":"reassign","task":task,"agent":agent})
        }
        Action::Reassign { .. } => return Err("unknown target agent".into()),
        Action::SetModel { task, model }
            if !model.is_empty()
                && report["plan"]["tasks"]
                    .as_array()
                    .and_then(|tasks| tasks.iter().find(|task_plan| task_plan["id"] == task))
                    .and_then(|task_plan| task_plan["agent"].as_str())
                    .and_then(|agent_id| {
                        report["plan"]["agents"]
                            .as_array()?
                            .iter()
                            .find(|profile| profile["id"] == agent_id)
                    })
                    .is_some_and(|profile| {
                        !profile["catalog"]["source"]
                            .as_str()
                            .unwrap_or_default()
                            .is_empty()
                            && profile["catalog"]["models"]
                                .as_array()
                                .is_some_and(|models| {
                                    models
                                        .iter()
                                        .any(|item| item.as_str() == Some(model.as_str()))
                                })
                    }) =>
        {
            serde_json::json!({"kind":"model","task":task,"model":model})
        }
        Action::SetModel { .. } => {
            return Err("model must belong to an observed CLI catalog".into());
        }
    };
    serde_json::to_vec(&serde_json::json!({
        "kind":"action", "revision":revision, "action":action
    }))
    .map_err(|error| error.to_string())
}
