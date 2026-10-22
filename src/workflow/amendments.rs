//! DD-179: plan amendments proposed by the coordinator. A deterministic validator decides what
//! may be applied; every amendment is an audit record with the values it replaced.
use super::*;
use serde_json::{Value, json};

/// Applied amendments allowed per run before a person decides.
pub const BUDGET: usize = 5;

pub(super) enum Outcome {
    /// The revision carries no amendment.
    None,
    Applied,
    /// A person must decide; the reason is shown and the plan is unchanged.
    Escalated(String),
}

fn applied_count(store: &Store) -> Result<usize, String> {
    Ok(store
        .audit("amendment", 1000)?
        .iter()
        .filter(|r| r["status"] == "applied")
        .count())
}

fn identity(task: &str, changes: &[Value], store: &Store) -> Result<String, String> {
    let seq = store.audit("amendment", 1000)?.len();
    Ok(format!(
        "am-{}",
        &digest(format!("{task}{changes:?}{seq}{}", now()).as_bytes())[..12]
    ))
}

/// Applies the amendment of one revision to `report.plan` when it is low risk and within
/// budget; otherwise records why a person must decide.
pub(super) fn apply(
    report: &mut Report,
    store: &mut Store,
    index: usize,
    revision: &crate::coordinator::Revision,
) -> Result<Outcome, String> {
    if revision.add_paths.is_empty() && revision.add_dependencies.is_empty() {
        return Ok(Outcome::None);
    }
    let task = report.plan.tasks[index].id.clone();
    let proposed =
        json!({"add_paths":revision.add_paths,"add_dependencies":revision.add_dependencies});
    let escalate = |store: &mut Store, reason: String| -> Result<Outcome, String> {
        let id = identity(&task, std::slice::from_ref(&proposed), store)?;
        store.record(
            "amendment",
            &json!({"id":id,"status":"escalated","task":task,"proposed":proposed,"reason":reason}),
            &format!("Amendment for task {task} needs a person: {reason}"),
        )?;
        Ok(Outcome::Escalated(reason))
    };
    if applied_count(store)? >= BUDGET {
        return escalate(
            store,
            format!("the amendment budget of {BUDGET} per run is used up"),
        );
    }
    let amended = match super::guardrail::amend(
        &report.plan,
        index,
        &revision.add_paths,
        &revision.add_dependencies,
    ) {
        Ok(amended) => amended,
        Err(reason) => return escalate(store, format!("refused by the plan rules: {reason}")),
    };
    if let Some(risk) = amended.risk {
        return escalate(store, risk);
    }
    // DD-180: measured against the charter recorded at the start of the run.
    let drift = store.optional_document("charter")?.map_or(0, |charter| {
        super::guardrail::drift(&charter, &amended.plan)
    });
    if drift > super::guardrail::DRIFT_LIMIT {
        return escalate(
            store,
            format!(
                "goal_drift: the task scopes would grow by {drift} paths beyond the charter (limit {})",
                super::guardrail::DRIFT_LIMIT
            ),
        );
    }
    let previous = json!({"paths":report.plan.tasks[index].paths,"depends_on":report.plan.tasks[index].depends_on});
    let id = identity(&task, &amended.changes, store)?;
    report.plan = amended.plan;
    store.record(
        "amendment",
        &json!({"id":id,"status":"applied","task":task,"changes":amended.changes,"previous":previous,"reason":revision.feedback,"drift":{"paths_added":drift,"limit":super::guardrail::DRIFT_LIMIT}}),
        &format!("Coordinator amended task {task}: {} (undo restores the earlier plan)", revision.feedback),
    )?;
    Ok(Outcome::Applied)
}

/// Restores what an applied amendment replaced. The run is left stopped so the operator
/// resumes explicitly; approvals are revoked.
pub fn amendment_undo(dir: &Path, id: &str) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    editable(&report)?;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    let records = store.audit("amendment", 1000)?;
    if records
        .iter()
        .any(|r| r["id"] == id && r["status"] == "undone")
    {
        return Err("this amendment was already undone".into());
    }
    let applied = records
        .iter()
        .rev()
        .find(|r| r["id"] == id && r["status"] == "applied")
        .ok_or("no applied amendment has this id")?;
    let task = applied["task"].as_str().unwrap_or_default().to_owned();
    let index = task_index(&report, &task)?;
    report.plan.tasks[index].paths =
        serde_json::from_value(applied["previous"]["paths"].clone()).map_err(|e| e.to_string())?;
    report.plan.tasks[index].depends_on =
        serde_json::from_value(applied["previous"]["depends_on"].clone())
            .map_err(|e| e.to_string())?;
    report.plan.validate()?;
    invalidate(&mut report, index, &mut store)?;
    report.state = "stopped".into();
    store.record(
        "amendment",
        &json!({"id":id,"status":"undone","task":task,"changes":applied["changes"],"previous":applied["previous"]}),
        &format!("Operator undid the amendment of task {task}; the earlier plan is restored"),
    )?;
    save(&mut store, dir, &report)
}
