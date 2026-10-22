//! DD-179: bounded, validated coordinator changes to a task contract.
use super::*;
use serde_json::{Value, json};

pub const BUDGET: usize = 5;

pub(super) enum Outcome {
    None,
    Applied,
    Escalated(String),
}

fn records(store: &Store) -> Result<Vec<Value>, String> {
    store.audit("amendment", 1000)
}

fn identity(task: &str, changes: &[Value], store: &Store) -> Result<String, String> {
    Ok(format!(
        "am-{}",
        &digest(format!("{task}{changes:?}{}{}", records(store)?.len(), now()).as_bytes())[..12]
    ))
}

pub(super) fn apply(
    report: &mut Report,
    store: &mut Store,
    index: usize,
    revision: &crate::coordinator::Revision,
) -> Result<Outcome, String> {
    if revision.add_paths.is_empty()
        && revision.add_dependencies.is_empty()
        && revision.test_command.is_none()
    {
        return Ok(Outcome::None);
    }
    let task = report.plan.tasks[index].id.clone();
    let proposed = json!({"add_paths":revision.add_paths,"add_dependencies":revision.add_dependencies,"test_command":revision.test_command});
    let escalate = |store: &mut Store, reason: String| -> Result<Outcome, String> {
        let id = identity(&task, std::slice::from_ref(&proposed), store)?;
        store.record(
            "amendment",
            &json!({"id":id,"status":"escalated","task":task,"proposed":proposed,"reason":reason}),
            &format!("Amendment for task {task} needs a person: {reason}"),
        )?;
        Ok(Outcome::Escalated(reason))
    };
    if records(store)?
        .iter()
        .filter(|r| r["status"] == "applied")
        .count()
        >= BUDGET
    {
        return escalate(
            store,
            format!("the amendment budget of {BUDGET} per run is used up"),
        );
    }
    let amended = match guardrail::amend_contract(
        &report.plan,
        &report.repo,
        index,
        &revision.add_paths,
        &revision.add_dependencies,
        revision.test_command.as_deref(),
    ) {
        Ok(amended) => amended,
        Err(reason) => return escalate(store, format!("refused by the plan rules: {reason}")),
    };
    if let Some(risk) = amended.risk {
        return escalate(store, risk);
    }
    let previous = json!({"paths":report.plan.tasks[index].paths,"depends_on":report.plan.tasks[index].depends_on,"test_command":report.plan.tasks[index].test_command});
    let id = identity(&task, &amended.changes, store)?;
    report.plan = amended.plan;
    report
        .charter
        .get_or_insert_with(|| guardrail::Charter::from_plan(&report.plan))
        .update_task(&report.plan, index)?;
    store.record("amendment", &json!({"id":id,"status":"applied","task":task,"changes":amended.changes,"previous":previous,"reason":revision.feedback}), &format!("Coordinator amended task {task}; operator can undo this change"))?;
    Ok(Outcome::Applied)
}

/// Undo a coordinator amendment, revoke derived approval and leave the run stopped.
pub fn amendment_undo(dir: &Path, id: &str) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    editable(&report)?;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    let rows = records(&store)?;
    if rows
        .iter()
        .any(|r| r["id"] == id && r["status"] == "undone")
    {
        return Err("this amendment was already undone".into());
    }
    let (position, applied) = rows
        .iter()
        .enumerate()
        .rev()
        .find(|(_, r)| r["id"] == id && r["status"] == "applied")
        .ok_or("no applied amendment has this id")?;
    let task = applied["task"].as_str().unwrap_or_default();
    if rows.iter().skip(position + 1).any(|later| {
        later["task"] == task
            && later["status"] == "applied"
            && !rows
                .iter()
                .any(|undo| undo["id"] == later["id"] && undo["status"] == "undone")
    }) {
        return Err("undo the later amendment for this task first".into());
    }
    let index = task_index(&report, task)?;
    report.plan.tasks[index].paths =
        serde_json::from_value(applied["previous"]["paths"].clone()).map_err(|e| e.to_string())?;
    report.plan.tasks[index].depends_on =
        serde_json::from_value(applied["previous"]["depends_on"].clone())
            .map_err(|e| e.to_string())?;
    if let Some(command) = applied["previous"].get("test_command") {
        report.plan.tasks[index].test_command =
            serde_json::from_value(command.clone()).map_err(|e| e.to_string())?;
    }
    report.plan.validate()?;
    report
        .charter
        .get_or_insert_with(|| guardrail::Charter::from_plan(&report.plan))
        .update_task(&report.plan, index)?;
    invalidate(&mut report, index, &mut store)?;
    report.state = "stopped".into();
    store.record("amendment", &json!({"id":id,"status":"undone","task":task,"changes":applied["changes"],"previous":applied["previous"]}), "Operator undid the amendment; earlier plan restored and approval revoked")?;
    save(&mut store, dir, &report)
}
