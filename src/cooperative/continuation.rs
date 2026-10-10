use super::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Continuation {
    pub directive_id: String,
    pub address: Address,
    pub prior_state: State,
    pub instruction: String,
    pub source: String,
}

/// Operator resume context only. Never acknowledges or reissues a directive.
pub fn prepare_continuation(
    store: &mut Store,
    task: &str,
    now: u64,
) -> Result<Option<Continuation>, String> {
    if now == 0 {
        return Err("invalid continuation observation time".into());
    }
    let tx = store
        .0
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    let mut r = report(&tx)?;
    let index = r
        .tasks
        .iter()
        .position(|t| t.id == task)
        .ok_or("unknown continuation task")?;
    if r.state != "stopped" || !matches!(r.tasks[index].status.as_str(), "stopped" | "blocked") {
        return Err("continuation requires explicitly resumed stopped work".into());
    }
    let profile = r
        .plan
        .agents
        .iter()
        .find(|p| p.id == r.plan.tasks[index].agent)
        .ok_or("unknown continuation profile")?;
    if profile.cooperative != crate::plan::CooperativeMode::CheckpointV1 {
        return Ok(None);
    }
    let namespace: Option<String> = tx
        .query_row(
            "SELECT value FROM documents WHERE key='cooperative_run_id'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let Some(namespace) = namespace else {
        return Ok(None);
    };
    // Build the existing attempt identity without pretending to launch or resume it.
    r.tasks[index].status = "running".into();
    let a = address(
        &r,
        task,
        serde_json::from_str(&namespace).map_err(|e| e.to_string())?,
    )?;
    let Some(mut d) = active(&tx, &a)? else {
        return Ok(None);
    };
    if d.address != a {
        return Err("continuation directive belongs to another assignment".into());
    }
    let key = format!("cooperative_continuation/{}/{}", a.task, a.attempt);
    let prior: Option<String> = tx
        .query_row("SELECT value FROM documents WHERE key=?1", [&key], |row| {
            row.get(0)
        })
        .optional()
        .map_err(|e| e.to_string())?;
    if let Some(value) = prior {
        let context: Continuation = serde_json::from_str(&value).map_err(|e| e.to_string())?;
        if d.state != State::Superseded
            || d.action != Action::Rework
            || context.directive_id != d.id
            || context.address != a
            || context.instruction != d.instruction
            || context.source != "local checkpoint ledger"
            || !matches!(context.prior_state, State::Pending | State::Accepted)
        {
            return Err("continuation receipt does not match original directive".into());
        }
        return Ok(Some(context));
    }
    expire(&tx, &mut d, now)?;
    let context = if d.action == Action::Rework
        && matches!(d.state, State::Pending | State::Accepted)
    {
        let context = Continuation {
            directive_id: d.id.clone(),
            address: a,
            prior_state: d.state.clone(),
            instruction: d.instruction.clone(),
            source: "local checkpoint ledger".into(),
        };
        if serde_json::to_vec_pretty(&context)
            .map_err(|e| e.to_string())?
            .len()
            > 16 * 1024
        {
            return Err("checkpoint continuation exceeds 16 KiB".into());
        }
        d.state = State::Superseded;
        save(&tx, &d)?;
        record_transaction(
            &tx,
            &key,
            &serde_json::to_value(&context).map_err(|e| e.to_string())?,
            "Explicit resume preserved unfinished local rework as context; not execution permission",
        )?;
        Some(context)
    } else {
        if matches!(d.state, State::Pending | State::Accepted) {
            d.state = State::Superseded;
            save(&tx, &d)?;
        }
        None
    };
    tx.commit().map_err(|e| e.to_string())?;
    Ok(context)
}
