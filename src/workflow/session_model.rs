use super::*;
use crate::plan::{CatalogOrigin, ModelCatalog};

/// Changes a task's model: queued for a pending task of a running run (#103), otherwise applied to the stopped run.
pub fn set_model(dir: &Path, id: &str, model: &str) -> Result<(), String> {
    if super::pending_edit::queue(dir, id, None, Some(model))? {
        return Ok(());
    }
    apply(dir, id, model, false)
}

/// GitHub #430: the operator enters a model id the CLI did not report. It joins the task agent's catalog as a manual
/// entry, then is validated and applied like any model; a rejected id changes nothing. Only an editable (stopped) run.
pub fn register_model(dir: &Path, id: &str, model: &str) -> Result<(), String> {
    apply(dir, id, model, true)
}

fn apply(dir: &Path, id: &str, model: &str, register: bool) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    editable(&report)?;
    let index = task_index(&report, id)?;
    let agent = report
        .plan
        .agents
        .iter_mut()
        .find(|a| a.id == report.plan.tasks[index].agent)
        .unwrap();
    if register {
        let mut candidate = agent.clone();
        let catalog = candidate.catalog.get_or_insert_with(|| ModelCatalog {
            models: Vec::new(),
            source: "Operator-configured model IDs; CLI discovery unavailable".into(),
            queried_at: now_seconds(),
            cli_version: "Not exposed by CLI".into(),
            origin: CatalogOrigin::Manual,
        });
        if !catalog.models.iter().any(|m| m == model) {
            catalog.models.push(model.into());
        }
        catalog.origin = CatalogOrigin::Manual;
        candidate.validate_model(model)?;
        *agent = candidate;
    } else {
        agent.validate_model(model)?;
    }
    report.plan.tasks[index].model = Some(model.into());
    let mut store = Store::open(dir.join("state.sqlite"))?;
    let previous_status = report.tasks[index].status.clone();
    invalidate(&mut report, index, &mut store)?;
    if previous_status == "waiting_availability" {
        report.tasks[index].status = previous_status;
        report.state = "waiting_availability".into();
    }
    store.record(
        "session_model",
        &serde_json::json!({"task":id,"model":model,"registered":register}),
        "Human changed session model; work must be verified again",
    )?;
    save(&mut store, dir, &report)
}

/// GitHub #1024: sets (or with `None` clears, back to the agent default) a task's effort. Only an editable (stopped)
/// run; the task must be verified again, as after a model change.
pub fn set_effort(dir: &Path, id: &str, effort: Option<&str>) -> Result<(), String> {
    if effort.is_some_and(|e| !crate::task_profile::EFFORTS.contains(&e)) {
        return Err("effort must be low, medium, high, xhigh or max".into());
    }
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    editable(&report)?;
    let index = task_index(&report, id)?;
    let options = &mut report.plan.tasks[index].execution_options;
    match effort {
        Some(effort) => options.insert("effort".into(), effort.into()),
        None => options.remove("effort"),
    };
    report.plan.validate()?;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    let previous_status = report.tasks[index].status.clone();
    invalidate(&mut report, index, &mut store)?;
    if previous_status == "waiting_availability" {
        report.tasks[index].status = previous_status;
        report.state = "waiting_availability".into();
    }
    store.record(
        "session_effort",
        &serde_json::json!({"task":id,"effort":effort}),
        "Human changed session effort; work must be verified again",
    )?;
    save(&mut store, dir, &report)
}

fn now_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}
