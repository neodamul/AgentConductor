//! DD-136: coordinator reservations live in run state, never in a page timer.
use super::*;
pub const FALLBACK_WAIT_SECONDS: i64 = 5 * 60 * 60;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvailabilityTimer {
    pub task: String,
    pub agent: String,
    pub quota_group: String,
    pub at: i64,
    pub kind: String,
}
pub(super) fn fallback_due(r: &availability::Restriction, now: i64) -> bool {
    match r.available_at {
        Some(at) => at > now && at.saturating_sub(r.observed_at) > FALLBACK_WAIT_SECONDS,
        None => now.saturating_sub(r.observed_at) >= FALLBACK_WAIT_SECONDS,
    }
}
fn fallback_wake(report: &Report, source: &str, current: i64) -> Option<i64> {
    let mut p = report.plan.agents.iter().find(|a| a.id == source)?;
    let mut visited = std::collections::HashSet::from([p.id.as_str()]);
    let mut resets = vec![];
    while let Some(policy) = &p.fallback {
        let target = report.plan.agents.iter().find(|a| a.id == policy.agent)?;
        if !visited.insert(target.id.as_str()) {
            break;
        }
        match report.account_restrictions.get(target.quota_key()) {
            None => return Some(current),
            Some(r) => {
                if let Some(at) = r.available_at {
                    if at <= current {
                        return Some(current);
                    }
                    resets.push(at);
                }
            }
        }
        p = target;
    }
    resets.into_iter().min()
}
pub fn availability_timer(report: &Report, now: i64) -> Vec<AvailabilityTimer> {
    if !matches!(report.state.as_str(), "waiting_availability" | "ready") {
        return vec![];
    }
    report
        .tasks
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            if t.status != "waiting_availability" {
                return None;
            }
            let r = t.restriction.as_ref()?;
            let p = report
                .plan
                .agents
                .iter()
                .find(|a| a.id == report.plan.tasks[i].agent)?;
            let fallback = p.fallback.is_some();
            let (kind, at) = if fallback && fallback_due(r, now) {
                ("fallback", fallback_wake(report, &p.id, now)?)
            } else if let Some(at) = r.available_at {
                if t.automatic_retries >= 3 {
                    return None;
                }
                ("resume", at.max(t.retry_not_before))
            } else if fallback {
                (
                    "fallback",
                    r.observed_at.saturating_add(FALLBACK_WAIT_SECONDS),
                )
            } else {
                return None;
            };
            Some(AvailabilityTimer {
                task: t.id.clone(),
                agent: p.id.clone(),
                quota_group: p.quota_key().into(),
                at,
                kind: kind.into(),
            })
        })
        .collect()
}
fn binding(report: &Report, i: usize) -> Result<String, String> {
    serde_json::to_string(&(
        report.tasks[i].id.as_str(),
        report.plan.tasks[i].agent.as_str(),
    ))
    .map_err(|e| e.to_string())
}
pub(super) fn sync_availability(
    report: &mut Report,
    dir: &Path,
    store: &mut Store,
    current: i64,
) -> Result<bool, String> {
    let before = serde_json::to_vec(report).map_err(|e| e.to_string())?;
    let history = store
        .optional_document("availability_history")?
        .unwrap_or(serde_json::json!({}));
    let bindings = (0..report.tasks.len())
        .map(|i| binding(report, i))
        .collect::<Result<Vec<_>, _>>()?;
    if !matches!(
        report.state.as_str(),
        "waiting_availability" | "ready" | "running"
    ) {
        return Ok(false);
    }
    for p in &report.plan.agents {
        let view = crate::quota::stored_for_run(store, p, current, dir)?;
        if let Some(mut observed) = crate::quota::restriction(&view, current) {
            if let Some(old) = report.account_restrictions.get(p.quota_key()) {
                if old.source == "manual_schedule" {
                    continue;
                }
                observed.observed_at = old.observed_at;
                if old.available_at.is_some_and(|at| at > current)
                    && observed.available_at.is_none()
                {
                    observed.available_at = old.available_at;
                }
            }
            report
                .account_restrictions
                .insert(p.quota_key().into(), observed);
        }
    }
    let active_coordinator = report.state == "running";
    for (i, t) in report.tasks.iter_mut().enumerate() {
        if t.status == "pending" && !active_coordinator {
            continue;
        }
        if !matches!(t.status.as_str(), "pending" | "waiting_availability") {
            continue;
        }
        let p = report
            .plan
            .agents
            .iter()
            .find(|a| a.id == report.plan.tasks[i].agent)
            .ok_or("missing task profile")?;
        if let Some(account) = report.account_restrictions.get(p.quota_key()) {
            if t.restriction
                .as_ref()
                .is_some_and(|r| r.source == "manual_schedule")
            {
                continue;
            }
            let mut r = account.clone();
            if let Some(old) = &t.restriction {
                r.observed_at = old.observed_at;
                if r.available_at.is_none() && old.available_at.is_some_and(|at| at > current) {
                    r.available_at = old.available_at;
                }
            }
            if let Some(first) = history[&bindings[i]].as_i64() {
                r.observed_at = r.observed_at.min(first);
            }
            t.restriction = Some(r);
            t.status = "waiting_availability".into();
        }
    }
    for (i, t) in report.tasks.iter().enumerate() {
        if let Some(r) = &t.restriction {
            let p = report
                .plan
                .agents
                .iter()
                .find(|a| a.id == report.plan.tasks[i].agent)
                .ok_or("missing task profile")?;
            if let Some(account) = report.account_restrictions.get_mut(p.quota_key()) {
                account.observed_at = account.observed_at.min(r.observed_at);
            }
        }
    }
    if report.state == "ready"
        && report
            .tasks
            .iter()
            .any(|t| t.status == "waiting_availability")
    {
        report.state = "waiting_availability".into();
    }
    let changed = before != serde_json::to_vec(report).map_err(|e| e.to_string())?;
    record_timer(store, dir, report, current)?;
    Ok(changed)
}
pub(super) fn record_timer(
    store: &mut Store,
    dir: &Path,
    report: &Report,
    now: i64,
) -> Result<(), String> {
    let previous = store
        .optional_document("availability_history")?
        .unwrap_or(serde_json::json!({}));
    let mut history = serde_json::Map::new();
    if !dir.join("stop.request").exists()
        && !matches!(
            report.state.as_str(),
            "completed" | "waiting_approval" | "approved" | "rejected"
        )
    {
        for (i, t) in report.tasks.iter().enumerate() {
            if matches!(
                t.status.as_str(),
                "waiting_availability" | "pending" | "running" | "stopped"
            ) {
                let key = binding(report, i)?;
                if let Some(first) = previous[&key]
                    .as_i64()
                    .or_else(|| t.restriction.as_ref().map(|r| r.observed_at))
                {
                    history.insert(key, first.into());
                }
            }
        }
    }
    let history = serde_json::Value::Object(history);
    if previous != history {
        store.record(
            "availability_history",
            &history,
            "Original restriction time retained across retries; reassign and Stop cancel it",
        )?;
    }
    let value = serde_json::json!({"schema":1,"entries":if dir.join("stop.request").exists(){vec![]}else{availability_timer(report,now)},"fallback_wait_seconds":FALLBACK_WAIT_SECONDS,"instruction":"reload run/assignment before acting; cancel reassign/Stop; resume once at due; only reviewed fallback after five-hour horizon"});
    if store.optional_document("availability_timer")?.as_ref() != Some(&value) {
        store.record("availability_timer",&value,"Coordinator availability reservation updated; assignments and Stop remain authoritative")?;
    }
    Ok(())
}
pub fn refresh_availability(dir: &Path, current: i64) -> Result<Vec<AvailabilityTimer>, String> {
    let _lock = match Lock::acquire(dir) {
        Ok(lock) => lock,
        Err(_) => return Ok(availability_timer(&load(dir)?, current)),
    };
    let mut report = load(dir)?;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    if dir.join("stop.request").exists()
        && matches!(report.state.as_str(), "ready" | "waiting_availability")
        && !report.tasks.iter().any(|t| t.status == "running")
    {
        report.state = "stopped".into();
        store.record(
            "approval",
            &serde_json::Value::Null,
            "Stop revoked candidate approval",
        )?;
        save(&mut store, dir, &report)?;
    }
    if sync_availability(&mut report, dir, &mut store, current)? {
        save(&mut store, dir, &report)?;
    }
    record_timer(&mut store, dir, &report, current)?;
    Ok(if dir.join("stop.request").exists() {
        vec![]
    } else {
        availability_timer(&report, current)
    })
}
