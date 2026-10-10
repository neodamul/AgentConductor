use super::*;

pub(super) fn editable(report: &Report) -> Result<(), String> {
    if matches!(report.state.as_str(), "running" | "completed")
        || report.tasks.iter().any(|t| t.status == "running")
    {
        return Err("stop the active coordinator before editing; merged runs are immutable".into());
    }
    Ok(())
}

pub(super) fn task_index(report: &Report, id: &str) -> Result<usize, String> {
    report
        .tasks
        .iter()
        .position(|t| t.id == id)
        .ok_or_else(|| "unknown task/session ID".into())
}

pub(super) fn invalidate(
    report: &mut Report,
    index: usize,
    store: &mut Store,
) -> Result<(), String> {
    report.candidate = None;
    report.diff_hash = None;
    report.tests.clear();
    report.error = None;
    store.record(
        "approval",
        &serde_json::Value::Null,
        "Session edit revoked prior integration approval",
    )?;
    report.tasks[index].status = "pending".into();
    report.tasks[index].approval_epoch = report.tasks[index]
        .approval_epoch
        .checked_add(1)
        .ok_or("approval epoch exhausted")?;
    report.tasks[index].tool_request = None;
    report.tasks[index].pending_tool = None;
    report.tasks[index].snapshot = None;
    let mut changed = vec![report.tasks[index].id.clone()];
    loop {
        let mut added = false;
        for (i, task) in report.plan.tasks.iter().enumerate() {
            if !changed.contains(&task.id) && task.depends_on.iter().any(|id| changed.contains(id))
            {
                changed.push(task.id.clone());
                report.tasks[i].status = "pending".into();
                report.tasks[i].approval_epoch = report.tasks[i]
                    .approval_epoch
                    .checked_add(1)
                    .ok_or("approval epoch exhausted")?;
                report.tasks[i].tool_request = None;
                report.tasks[i].pending_tool = None;
                report.tasks[i].snapshot = None;
                added = true;
            }
        }
        if !added {
            break;
        }
    }
    report.state = "ready".into();
    Ok(())
}

/// Scheduling a session also applies to waiting sessions in its account quota group.
pub fn schedule(dir: &Path, id: &str, timestamp: &str) -> Result<(), String> {
    let time = availability::parse_time(timestamp)?;
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    editable(&report)?;
    let index = task_index(&report, id)?;
    if report.tasks[index].status != "waiting_availability" {
        return Err("session has no temporary restriction".into());
    }
    let profile = report
        .plan
        .agents
        .iter()
        .find(|a| a.id == report.plan.tasks[index].agent)
        .unwrap();
    let group = profile.quota_key().to_string();
    for (i, task) in report.tasks.iter_mut().enumerate() {
        let agent = report
            .plan
            .agents
            .iter()
            .find(|a| a.id == report.plan.tasks[i].agent)
            .unwrap();
        if agent.quota_key() == group
            && task.status == "waiting_availability"
            && let Some(restriction) = &mut task.restriction
        {
            restriction.available_at = Some(time);
            restriction.source = "manual_schedule".into();
            task.retry_not_before = time;
            task.automatic_retries = 0;
        }
    }
    if let Some(restriction) = report.account_restrictions.get_mut(&group) {
        restriction.available_at = Some(time);
        restriction.source = "manual_schedule".into();
    }
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record("schedule",&serde_json::json!({"task":id,"quota_group":group,"available_at":time,"original_timestamp":timestamp}),"Human scheduled restricted account sessions")?;
    save(&mut store, dir, &report)
}
