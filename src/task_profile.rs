//! GitHub #210: model and effort follow the kind of task and what the model honors, instead of being the same for
//! every task. The operator's explicit choice always wins; the agent's own `task_defaults` come before the built-in
//! table; an effort the model does not honor is lowered with a warning, never silently and never as a block.
use crate::plan::{AgentProfile, Task, TaskType};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Warning {
    /// The effort that fits the task type is not honored by the model; `applied` is what runs instead.
    EffortUnsupported {
        asked: String,
        applied: Option<String>,
    },
    /// The agent declares that the model is not a good fit for this kind of task.
    ModelNotSuited { model: String, task_type: TaskType },
}

#[derive(Debug, Clone, Serialize)]
pub struct Resolved {
    pub task_type: TaskType,
    pub model: Option<String>,
    /// The task's execution options, plus the effort resolved for it when the operator chose none.
    pub options: BTreeMap<String, String>,
    pub warnings: Vec<Warning>,
}

const EFFORTS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];

fn name(kind: TaskType) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

/// The default effort of each kind (docs/cli-io-protocol.md §2.1).
fn built_in(kind: TaskType) -> &'static str {
    match kind {
        TaskType::Extract => "low",
        TaskType::Summarize | TaskType::Analyze | TaskType::Creative => "medium",
        TaskType::Code | TaskType::Agentic => "high",
    }
}

/// The highest allowed effort not above `asked`; when every allowed value is higher, the lowest one.
fn nearest(allowed: &[String], asked: &str) -> Option<String> {
    let rank = |v: &str| EFFORTS.iter().position(|e| *e == v);
    let want = rank(asked)?;
    let known: Vec<(usize, &String)> = allowed.iter().filter_map(|v| Some((rank(v)?, v))).collect();
    known
        .iter()
        .filter(|(r, _)| *r <= want)
        .max_by_key(|(r, _)| *r)
        .or_else(|| known.iter().min_by_key(|(r, _)| *r))
        .map(|(_, v)| (*v).clone())
}

pub fn resolve(profile: &AgentProfile, task: &Task) -> Result<Resolved, String> {
    let task_type = task.task_type.unwrap_or(TaskType::Code);
    let settings =
        crate::agent_settings::settings_for(profile.parser, profile.agent_settings.as_ref())?;
    let capabilities = settings.as_ref().map(|s| &s.capabilities);
    let defaults = capabilities.map(|c| &c["task_defaults"][name(task_type)]);
    let model = task.model.clone().or_else(|| {
        let recommended = defaults?["model"].as_str()?;
        profile
            .validate_model(recommended)
            .ok()
            .map(|_| recommended.to_string())
    });
    let mut options = task.execution_options.clone();
    let mut warnings = vec![];
    if let Some(group) = profile
        .execution_options
        .iter()
        .find(|o| o.name == "effort")
        && !options.contains_key("effort")
    {
        let asked = defaults
            .and_then(|d| d["effort"].as_str())
            .unwrap_or(built_in(task_type))
            .to_string();
        // Without a chosen model a model-specific group says nothing yet, so nothing is applied.
        let allowed = if model.is_none() && !group.model_values.is_empty() {
            None
        } else {
            Some(
                group
                    .allowed_values(model.as_deref())
                    .cloned()
                    .unwrap_or_default(),
            )
        };
        if let Some(allowed) = allowed {
            if allowed.contains(&asked) {
                options.insert("effort".into(), asked);
            } else {
                let applied = nearest(&allowed, &asked);
                if let Some(value) = &applied {
                    options.insert("effort".into(), value.clone());
                }
                warnings.push(Warning::EffortUnsupported { asked, applied });
            }
        }
    }
    if let (Some(model), Some(models)) = (&model, capabilities.and_then(|c| c["models"].as_array()))
        && let Some(best) = models
            .iter()
            .find(|m| m["id"] == model.as_str())
            .and_then(|m| m["traits"]["best_for"].as_array())
        && !best.is_empty()
        && !best.iter().any(|b| *b == name(task_type).as_str())
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
