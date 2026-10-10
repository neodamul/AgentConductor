use super::*;

pub fn revise(dir: &Path, id: &str, feedback: &str) -> Result<(), String> {
    if feedback.trim().is_empty() || feedback.len() > 16 * 1024 {
        return Err("review feedback must contain 1..16384 bytes".into());
    }
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    editable(&report)?;
    let index = task_index(&report, id)?;
    let previous = report.tasks[index].artifacts.clone();
    report.tasks[index].feedback = Some(feedback.into());
    report.tasks[index].restriction = None;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record("review_feedback", &serde_json::json!({"task":id,"feedback":feedback,"previous_artifacts":previous,"previous_session":report.tasks[index].session_id}), "Human requested scoped rework; original task intent retained")?;
    invalidate(&mut report, index, &mut store)?;
    save(&mut store, dir, &report)
}

pub fn set_model(dir: &Path, id: &str, model: &str) -> Result<(), String> {
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

pub fn reassign(dir: &Path, id: &str, agent: &str, model: Option<&str>) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    editable(&report)?;
    let index = task_index(&report, id)?;
    let target = report
        .plan
        .agents
        .iter()
        .find(|a| a.id == agent)
        .ok_or("unknown target agent")?;
    if let Some(model) = model {
        target.validate_model(model)?;
    }
    let mut store = Store::open(dir.join("state.sqlite"))?;
    transfer_role(
        &mut report,
        dir,
        index,
        agent,
        model,
        &mut store,
        "Human reassigned a role; target uses a new provider session",
    )?;
    save(&mut store, dir, &report)
}

pub(super) fn transfer_role(
    report: &mut Report,
    dir: &Path,
    index: usize,
    agent: &str,
    model: Option<&str>,
    store: &mut Store,
    note: &str,
) -> Result<(), String> {
    let id = report.tasks[index].id.clone();
    let old_agent = report.plan.tasks[index].agent.clone();
    let task = &report.tasks[index];
    let git = Repository::open(&task.cwd)?;
    let allowed = if report.plan.workspace_mode == WorkspaceMode::Shared {
        scopes(&report.plan)
    } else {
        report.plan.tasks[index].paths.clone()
    };
    git.check_scope(&git.changed(&task.base)?, &allowed)?;
    let mut args = vec!["diff", "--binary", task.base.as_str(), "--"];
    args.extend(report.plan.tasks[index].paths.iter().map(String::as_str));
    let diff = git.command(&args)?;
    let handoff = dir.join(format!("handoff-{index}-{}.md", task.attempt));
    let text = format!(
        "# Role handoff\n\nGoal: {}\nTask: {}\nFrom: {}\nTo: {}\nPrevious provider session: {:?}\nOwned paths: {}\nPrior evidence: {}\n\nInspect existing owned files, including untracked files, before continuing. Preserve completed work.\n\n```diff\n{}\n```\n",
        report.plan.goal,
        id,
        old_agent,
        agent,
        task.session_id,
        report.plan.tasks[index].paths.join(", "),
        task.artifacts.display(),
        String::from_utf8_lossy(&diff[..diff.len().min(64 * 1024)])
    );
    fs::write(&handoff, &text).map_err(|e| e.to_string())?;
    store.record("handoff",&serde_json::json!({"task":id,"from":old_agent,"to":agent,"previous_session":task.session_id,"evidence":task.artifacts,"handoff":handoff}),note)?;
    report.plan.tasks[index].agent = agent.into();
    report.plan.tasks[index].model = model.map(String::from);
    // Bounded facts only; no free-form cross-agent conversation or foreign native ID.
    report.plan.tasks[index].prompt.push_str("\n\n");
    report.plan.tasks[index].prompt.push_str(&text);
    report.tasks[index].session_id = None;
    report.tasks[index].session_established = false;
    report.tasks[index].restriction = None;
    invalidate(report, index, store)?;
    Ok(())
}

/// Called only after JoinSet is empty. Reviewed policies never steer a live worker.
pub(super) fn apply_fallbacks(
    report: &mut Report,
    dir: &Path,
    store: &mut Store,
) -> Result<bool, String> {
    let may_transfer = report.tasks.iter().enumerate().any(|(index, task)| {
        matches!(task.status.as_str(), "pending" | "waiting_availability")
            && report
                .plan
                .agents
                .iter()
                .find(|agent| agent.id == report.plan.tasks[index].agent)
                .is_some_and(|agent| {
                    agent.fallback.is_some()
                        && report.account_restrictions.contains_key(agent.quota_key())
                })
    });
    if may_transfer {
        // A completed leader is insufficient: surviving descendants must also be absent.
        for task in &report.tasks {
            let path = task.artifacts.join("process/owner.json");
            if !path.exists() && task.attempt > 0 {
                return Err("missing worker ownership evidence; fallback refused".into());
            }
            if path.exists() {
                let owner: process::Ownership =
                    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?;
                owner.ensure_quiescent()?;
            }
        }
    }
    let mut changed = false;
    for index in 0..report.tasks.len() {
        if !matches!(
            report.tasks[index].status.as_str(),
            "pending" | "waiting_availability"
        ) {
            continue;
        }
        let source = report
            .plan
            .agents
            .iter()
            .find(|agent| agent.id == report.plan.tasks[index].agent)
            .unwrap();
        let Some(restriction) = report.account_restrictions.get(source.quota_key()).cloned() else {
            continue;
        };
        report.tasks[index].status = "waiting_availability".into();
        report.tasks[index].restriction = Some(restriction);
        let mut profile = source;
        let mut selected = None;
        while let Some(policy) = &profile.fallback {
            let target = report
                .plan
                .agents
                .iter()
                .find(|agent| agent.id == policy.agent)
                .ok_or("unknown fallback agent")?;
            if !report.account_restrictions.contains_key(target.quota_key()) {
                selected = Some((target.id.clone(), policy.model.clone()));
                break;
            }
            profile = target;
        }
        if let Some((agent, model)) = selected {
            transfer_role(
                report,
                dir,
                index,
                &agent,
                model.as_deref(),
                store,
                "Reviewed fallback transferred a restricted role; target uses a new provider session",
            )?;
            changed = true;
        }
    }
    Ok(changed)
}

/// DD-43: called only after JoinSet is empty. A completed handoff continues on the first
/// unrestricted target of its reviewed fallback chain once every owned process is absent;
/// otherwise, or when a stop was requested, it remains stopped for explicit action.
pub(super) fn apply_handoffs(
    report: &mut Report,
    dir: &Path,
    store: &mut Store,
) -> Result<bool, String> {
    let handoffs: Vec<usize> = (0..report.tasks.len())
        .filter(|&i| report.tasks[i].status == "handoff")
        .collect();
    if handoffs.is_empty() {
        return Ok(false);
    }
    let quiescent = report.tasks.iter().all(|task| {
        let path = task.artifacts.join("process/owner.json");
        if !path.exists() {
            return task.attempt == 0;
        }
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<process::Ownership>(&bytes).ok())
            .is_some_and(|owner| owner.ensure_quiescent().is_ok())
    });
    let mut changed = false;
    for index in handoffs {
        report.tasks[index].status = "stopped".into();
        if !quiescent || dir.join("stop.request").exists() {
            continue;
        }
        let mut profile = report
            .plan
            .agents
            .iter()
            .find(|agent| agent.id == report.plan.tasks[index].agent)
            .ok_or("unknown task agent")?;
        let mut selected = None;
        while let Some(policy) = &profile.fallback {
            let target = report
                .plan
                .agents
                .iter()
                .find(|agent| agent.id == policy.agent)
                .ok_or("unknown fallback agent")?;
            if !report.account_restrictions.contains_key(target.quota_key()) {
                selected = Some((target.id.clone(), policy.model.clone()));
                break;
            }
            profile = target;
        }
        if let Some((agent, model)) = selected {
            transfer_role(
                report,
                dir,
                index,
                &agent,
                model.as_deref(),
                store,
                "Completed checkpoint handoff transferred to the reviewed fallback; target uses a new provider session",
            )?;
            changed = true;
        }
    }
    Ok(changed)
}
