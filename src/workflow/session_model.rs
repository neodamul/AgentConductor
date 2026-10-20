use super::*;

/// Changes a task's model: queued for a pending task of a running run (#103), otherwise applied to the stopped run.
pub fn set_model(dir: &Path, id: &str, model: &str) -> Result<(), String> {
    if super::pending_edit::queue(dir, id, None, Some(model))? {
        return Ok(());
    }
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    editable(&report)?;
    let index = task_index(&report, id)?;
    report
        .plan
        .agents
        .iter()
        .find(|a| a.id == report.plan.tasks[index].agent)
        .unwrap()
        .validate_model(model)?;
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
        &serde_json::json!({"task":id,"model":model}),
        "Human changed session model; work must be verified again",
    )?;
    save(&mut store, dir, &report)
}
