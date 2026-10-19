use super::{Binding, decision_wait, orchestrator::orchestrate};
use crate::{claude_control::ToolDecision, gates::ToolRequest};

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Creates the human-visible exact request before consulting the coordinator.
pub(super) async fn coordinated_decision(
    binding: &Binding,
    request: &ToolRequest,
    session: &str,
    stop: &std::path::Path,
) -> Result<Option<ToolDecision>, String> {
    let engine = std::env::current_exe().map_err(|e| e.to_string())?;
    let context =
        crate::workflow::verify_tool_context(&binding.dir, &binding.task, request, &engine).await?;
    let db = binding.dir.join("state.sqlite");
    let id = crate::gates::Ledger::open(&db)?.request(&context, request, unix_now())?;
    crate::state::Store::open(&db)?.record(
        "tool_live_wait",
        &serde_json::json!({"id":id,"task":binding.task}),
        "Worker paused before execution for an exact human tool decision",
    )?;

    let mut coordinator = Box::pin(orchestrate(binding, request, stop));
    let wait = decision_wait();
    let mut human = Box::pin(wait_for_live_decision(
        binding, request, session, stop, None,
    ));
    tokio::select! {
        biased;
        decision = &mut human => match decision? {
            Some(decision) => Ok(Some(decision)),
            None => match coordinator.await {
                Some(decision) => decide_coordinator(binding, request, session, decision, &engine).await,
                None => wait_for_live_decision(binding, request, session, stop, Some(wait)).await,
            },
        },
        decision = &mut coordinator => match decision {
            Some(decision) => match decide_coordinator(binding, request, session, decision, &engine).await? {
                Some(decision) => Ok(Some(decision)),
                None => wait_for_live_decision(binding, request, session, stop, Some(wait)).await,
            },
            None if wait == 0 => Ok(None),
            None => wait_for_live_decision(binding, request, session, stop, Some(wait)).await,
        },
    }
}

/// The first transaction to decide the visible receipt wins.
async fn decide_coordinator(
    binding: &Binding,
    request: &ToolRequest,
    session: &str,
    decision: ToolDecision,
    engine: &std::path::Path,
) -> Result<Option<ToolDecision>, String> {
    let db = binding.dir.join("state.sqlite");
    let Some(receipt) = crate::gates::Ledger::open(&db)?
        .requests()?
        .into_iter()
        .find(|receipt| {
            receipt.context.task == binding.task
                && receipt.context.session == session
                && receipt.request == *request
                && receipt.status != "superseded"
        })
    else {
        return Ok(None);
    };
    let applied = receipt.status == "pending"
        && crate::workflow::decide_live_request(
            &binding.dir,
            &receipt.id,
            decision.allow,
            300,
            engine,
        )
        .await
        .is_ok();
    let receipt = crate::gates::Ledger::open(&db)?
        .requests()?
        .into_iter()
        .find(|receipt| {
            receipt.context.task == binding.task
                && receipt.context.session == session
                && receipt.request == *request
                && receipt.status != "superseded"
        })
        .ok_or("live tool receipt disappeared")?;
    match receipt.status.as_str() {
        "approved" => {
            let human = consume_live_approval(binding, request, &receipt, engine).await?;
            Ok(human.map(|human| if applied { decision } else { human }))
        }
        "consumed" => Ok(Some(ToolDecision {
            allow: true,
            reason: "Exact live human approval consumed once".into(),
        })),
        "denied" if applied => Ok(Some(decision)),
        "denied" => Ok(Some(ToolDecision::deny(
            "A human denied this exact tool request",
        ))),
        _ => Ok(None),
    }
}

async fn consume_live_approval(
    binding: &Binding,
    request: &ToolRequest,
    receipt: &crate::gates::Receipt,
    engine: &std::path::Path,
) -> Result<Option<ToolDecision>, String> {
    let fresh =
        crate::workflow::verify_tool_context(&binding.dir, &binding.task, request, engine).await?;
    let bound = crate::gates::Context {
        workspace_sha256: receipt.context.workspace_sha256.clone(),
        ..fresh
    };
    let consumed = crate::gates::Ledger::open(&binding.dir.join("state.sqlite"))?.consume(
        &receipt.id,
        &bound,
        request,
        unix_now(),
    )?;
    Ok(
        (consumed == crate::gates::Decision::Allowed).then(|| ToolDecision {
            allow: true,
            reason: "Exact live human approval consumed once".into(),
        }),
    )
}

/// Waits on a pre-created exact request; `None` duration waits until the coordinator replies.
async fn wait_for_live_decision(
    binding: &Binding,
    request: &ToolRequest,
    session: &str,
    stop: &std::path::Path,
    timeout: Option<u64>,
) -> Result<Option<ToolDecision>, String> {
    let engine = std::env::current_exe().map_err(|e| e.to_string())?;
    let db = binding.dir.join("state.sqlite");
    let deadline =
        timeout.map(|wait| std::time::Instant::now() + std::time::Duration::from_secs(wait));
    loop {
        if stop.exists() {
            return Err("Human stop while awaiting a tool decision".into());
        }
        if deadline.is_some_and(|deadline| std::time::Instant::now() >= deadline) {
            return Ok(None);
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        let latest = crate::gates::Ledger::open(&db)?
            .requests()?
            .into_iter()
            .find(|r| {
                r.context.task == binding.task
                    && r.context.session == session
                    && r.request == *request
                    && r.status != "superseded"
            });
        let Some(latest) = latest else { continue };
        match latest.status.as_str() {
            "denied" => {
                return Ok(Some(ToolDecision::deny(
                    "A human denied this exact tool request",
                )));
            }
            "approved" => {
                if let Some(decision) =
                    consume_live_approval(binding, request, &latest, &engine).await?
                {
                    return Ok(Some(decision));
                }
                let fresh = crate::workflow::verify_tool_context(
                    &binding.dir,
                    &binding.task,
                    request,
                    &engine,
                )
                .await?;
                let mut ledger = crate::gates::Ledger::open(&db)?;
                ledger.supersede(&latest.id)?;
                ledger.request(&fresh, request, unix_now())?;
            }
            _ => {}
        }
    }
}
