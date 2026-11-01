use crate::plan::{AgentProfile, Task, TaskType};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Warning {
    EffortUnsupported {
        asked: String,
        applied: Option<String>,
    },
    ModelNotSuited {
        model: String,
        task_type: TaskType,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct Resolved {
    pub task_type: TaskType,
    pub model: Option<String>,
    pub options: BTreeMap<String, String>,
    pub warnings: Vec<Warning>,
}

const EFFORTS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];

fn default_effort(kind: TaskType) -> &'static str {
    match kind {
        TaskType::Extract => "low",
        TaskType::Summarize | TaskType::Analyze | TaskType::Creative => "medium",
        TaskType::Code | TaskType::Agentic => "high",
    }
}

fn task_name(kind: TaskType) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
}

fn nearest(values: &[String], asked: &str) -> Option<String> {
    let rank = |value: &str| EFFORTS.iter().position(|effort| *effort == value);
    let want = rank(asked)?;
    let known: Vec<_> = values
        .iter()
        .filter_map(|value| Some((rank(value)?, value)))
        .collect();
    known
        .iter()
        .filter(|(order, _)| *order <= want)
        .max_by_key(|(order, _)| *order)
        .or_else(|| known.iter().min_by_key(|(order, _)| *order))
        .map(|(_, value)| (*value).clone())
}

pub fn resolve(profile: &AgentProfile, task: &Task) -> Result<Resolved, String> {
    let task_type = task.task_type.unwrap_or(TaskType::Code);
    let capabilities = profile
        .agent_settings
        .as_ref()
        .map(|settings| &settings["capabilities"]);
    let defaults = capabilities.map(|value| &value["task_defaults"][task_name(task_type)]);
    let model = task.model.clone().or_else(|| {
        let recommended = defaults?.get("model")?.as_str()?;
        profile
            .validate_model(recommended)
            .ok()
            .map(|_| recommended.to_owned())
    });
    let mut options = task.execution_options.clone();
    let mut warnings = Vec::new();
    if let Some(group) = profile
        .execution_options
        .iter()
        .find(|group| group.name == "effort")
        && !options.contains_key("effort")
    {
        let values = if model.is_none() && !group.model_values.is_empty() {
            None
        } else {
            group.allowed_values(model.as_deref()).cloned()
        };
        if let Some(values) = values {
            let asked = defaults
                .and_then(|value| value.get("effort"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| default_effort(task_type))
                .to_owned();
            if values.contains(&asked) {
                options.insert("effort".into(), asked);
            } else {
                let applied = nearest(&values, &asked);
                if let Some(value) = &applied {
                    options.insert("effort".into(), value.clone());
                }
                warnings.push(Warning::EffortUnsupported { asked, applied });
            }
        }
    }
    if let (Some(model), Some(models)) = (
        &model,
        capabilities.and_then(|value| value["models"].as_array()),
    ) && let Some(best_for) = models
        .iter()
        .find(|entry| entry["id"] == model.as_str())
        .and_then(|entry| entry["traits"]["best_for"].as_array())
        && !best_for.is_empty()
        && !best_for.iter().any(|kind| kind == &task_name(task_type))
    {
        warnings.push(Warning::ModelNotSuited {
            model: model.clone(),
            task_type,
        });
    }
    Ok(Resolved {
        task_type,
        model,
        options,
        warnings,
    })
}

pub fn summaries(profiles: &[AgentProfile], tasks: &[Task]) -> Vec<serde_json::Value> {
    tasks
        .iter()
        .map(|task| {
            let resolved = profiles
                .iter()
                .find(|profile| profile.id == task.agent)
                .ok_or_else(|| format!("agent {} is not registered", task.agent))
                .and_then(|profile| resolve(profile, task));
            match resolved {
                Ok(value) => serde_json::json!({
                    "task_id": task.id,
                    "model": value.model,
                    "options": value.options,
                    "warnings": value.warnings,
                }),
                Err(error) => serde_json::json!({"task_id": task.id, "error": error}),
            }
        })
        .collect()
}
