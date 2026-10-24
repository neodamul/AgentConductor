use eframe::egui::{self, Ui};
use serde_json::Value;

pub(super) fn model_picker(ui: &mut Ui, language: &str, task: &mut Value, agents: &[Value]) {
    let text = |key| crate::ui::messages::in_language(language, key);
    let agent = agents.iter().find(|agent| agent["id"] == task["agent"]);
    let models = agent
        .and_then(|agent| agent["catalog"]["models"].as_array())
        .cloned()
        .unwrap_or_default();
    let selected = agent
        .map(|agent| task_model_selection_label(task, agent))
        .unwrap_or_else(|| text("startup.automatic").to_owned());
    let response = egui::ComboBox::from_id_salt(("task-model", task["id"].as_str().unwrap_or("")))
        .selected_text(&selected)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(task_model(task, agent).is_none(), text("startup.automatic"))
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
    response
        .response
        .accessible_name(crate::ui::messages::in_language(
            language,
            "startup.session_model",
        ));
}

pub(super) fn execution_option_pickers(
    ui: &mut Ui,
    language: &str,
    task: &mut Value,
    agents: &[Value],
) {
    let default_label = crate::ui::messages::in_language(language, "startup.profile_default");
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
            .unwrap_or(default_label);
        let response =
            egui::ComboBox::from_id_salt(("task-option", task["id"].as_str().unwrap_or(""), name))
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(task["execution_options"][name].is_null(), default_label)
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
        response
            .response
            .accessible_name(crate::ui::messages::interpolate(
                crate::ui::messages::in_language(language, "startup.task_option_named"),
                &[("value1", name)],
            ));
    }
}

pub fn task_model_selection_label(task: &Value, agent: &Value) -> String {
    match task_model(task, Some(agent)) {
        Some(model) if has_task_model_override(task) => model.to_owned(),
        Some(model) => format!("{model} (profile default)"),
        None => "Automatic".to_owned(),
    }
}

pub fn task_option_values(task: &Value, agent: &Value, name: &str) -> Option<Vec<String>> {
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
