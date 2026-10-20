use eframe::egui::{self, Ui};
use serde_json::Value;

pub(super) fn text_field(ui: &mut Ui, label: &str, task: &mut Value, key: &str) {
    ui.label(label);
    let mut value = match key {
        "paths" | "depends_on" => task[key]
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(if key == "paths" { "\n" } else { ", " })
            })
            .unwrap_or_default(),
        "test_command" => task[key]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| serde_json::to_string(&task[key]).unwrap_or_else(|_| "[]".into())),
        _ => task[key].as_str().unwrap_or_default().to_owned(),
    };
    let response = if key == "prompt" {
        ui.add(egui::TextEdit::multiline(&mut value).desired_rows(3))
    } else {
        ui.add(egui::TextEdit::singleline(&mut value))
    };
    let changed = response.changed();
    response.accessible_name(label);
    if changed {
        task[key] = match key {
            "paths" => serde_json::json!(
                value
                    .lines()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            ),
            "depends_on" => serde_json::json!(
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            ),
            "test_command" => serde_json::from_str(&value).unwrap_or(Value::String(value)),
            _ => Value::String(value),
        };
    }
}

pub(super) fn agent_picker(ui: &mut Ui, task: &mut Value, agents: &[Value]) {
    ui.label("Assigned agent");
    let response = egui::ComboBox::from_id_salt(("task-agent", task["id"].as_str().unwrap_or("")))
        .selected_text(task["agent"].as_str().unwrap_or("Select agent"))
        .show_ui(ui, |ui| {
            for agent in agents {
                if let Some(id) = agent["id"].as_str() {
                    ui.selectable_value(&mut task["agent"], Value::String(id.into()), id);
                }
            }
        });
    response.response.accessible_name("Assigned agent");
}

pub(super) fn model_picker(ui: &mut Ui, task: &mut Value, agents: &[Value]) {
    let agent = agents.iter().find(|agent| agent["id"] == task["agent"]);
    let models = agent
        .and_then(|agent| agent["catalog"]["models"].as_array())
        .cloned()
        .unwrap_or_default();
    let selected = agent
        .map(|agent| task_model_selection_label(task, agent))
        .unwrap_or_else(|| "Automatic".to_owned());
    let response = egui::ComboBox::from_id_salt(("task-model", task["id"].as_str().unwrap_or("")))
        .selected_text(&selected)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(task_model(task, agent).is_none(), "Automatic")
                .clicked()
            {
                if supports_model_option(agent) {
                    remove_execution_option(task, "model");
                }
                task["model"] = Value::Null;
            }
            for model in models.iter().filter_map(Value::as_str) {
                if ui
                    .selectable_label(task_model(task, agent) == Some(model), model)
                    .clicked()
                {
                    if supports_model_option(agent) {
                        set_execution_option(task, "model", Value::String(model.into()));
                        task["model"] = Value::Null;
                    } else {
                        task["model"] = Value::String(model.into());
                    }
                }
            }
        });
    response.response.accessible_name("Session model");
}

pub(super) fn execution_option_pickers(ui: &mut Ui, task: &mut Value, agents: &[Value]) {
    let Some(agent) = agents.iter().find(|agent| agent["id"] == task["agent"]) else {
        return;
    };
    let Some(options) = agent["execution_options"].as_array() else {
        return;
    };
    for option in options {
        let Some(name) = option["name"].as_str() else {
            continue;
        };
        if matches!(name, "model" | "resume" | "session") {
            continue;
        }
        let Some(values) = task_option_values(task, agent, name) else {
            continue;
        };
        let selected = task["execution_options"][name]
            .as_str()
            .or(option["default"].as_str())
            .unwrap_or("Profile default");
        let response =
            egui::ComboBox::from_id_salt(("task-option", task["id"].as_str().unwrap_or(""), name))
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(
                            task["execution_options"][name].is_null(),
                            "Profile default",
                        )
                        .clicked()
                        && let Some(options) = task["execution_options"].as_object_mut()
                    {
                        options.remove(name);
                    }
                    for value in values.iter().map(String::as_str) {
                        if ui
                            .selectable_label(task["execution_options"][name] == value, value)
                            .clicked()
                        {
                            task["execution_options"][name] = value.into();
                        }
                    }
                });
        response.response.accessible_name(format!("Task {name}"));
    }
}

pub(super) fn task_model_selection_label(task: &Value, agent: &Value) -> String {
    match task_model(task, Some(agent)) {
        Some(model) if has_task_model_override(task) => model.to_owned(),
        Some(model) => format!("{model} (profile default)"),
        None => "Automatic".to_owned(),
    }
}

pub(super) fn task_option_values(task: &Value, agent: &Value, name: &str) -> Option<Vec<String>> {
    let option = agent["execution_options"]
        .as_array()?
        .iter()
        .find(|option| option["name"] == name)?;
    let model_values = option["model_values"].as_object();
    if model_values.is_some_and(|values| !values.is_empty()) {
        let model = task_model(task, Some(agent))?;
        return model_values?
            .get(model)?
            .as_array()?
            .iter()
            .map(|value| value.as_str().map(str::to_owned))
            .collect();
    }
    option["values"]
        .as_array()?
        .iter()
        .map(|value| value.as_str().map(str::to_owned))
        .collect()
}

fn task_model<'a>(task: &'a Value, agent: Option<&'a Value>) -> Option<&'a str> {
    task["execution_options"]["model"]
        .as_str()
        .or_else(|| task["model"].as_str())
        .or_else(|| {
            agent?["execution_options"]
                .as_array()?
                .iter()
                .find(|option| option["name"] == "model")?["default"]
                .as_str()
        })
}

fn has_task_model_override(task: &Value) -> bool {
    task["execution_options"]["model"].as_str().is_some() || task["model"].as_str().is_some()
}

fn supports_model_option(agent: Option<&Value>) -> bool {
    agent.is_some_and(|agent| {
        agent["execution_options"]
            .as_array()
            .is_some_and(|options| options.iter().any(|option| option["name"] == "model"))
    })
}

fn set_execution_option(task: &mut Value, name: &str, value: Value) {
    if !task["execution_options"].is_object() {
        task["execution_options"] = serde_json::json!({});
    }
    task["execution_options"][name] = value;
}

fn remove_execution_option(task: &mut Value, name: &str) {
    if let Some(options) = task["execution_options"].as_object_mut() {
        options.remove(name);
    }
}
