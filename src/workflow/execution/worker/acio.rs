use super::*;
use crate::process::ProcessResult;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

pub(super) struct Outcome {
    pub process: Result<ProcessResult, String>,
    pub pending: Option<crate::gates::ToolRequest>,
}

pub(super) fn input(
    task: &Task,
    report: &TaskReport,
    profile: &crate::task_profile::Resolved,
    cwd: &Path,
    prompt: &str,
    timeout: u64,
) -> Result<Vec<u8>, String> {
    let session = report
        .session_id
        .as_deref()
        .ok_or("owned session missing")?;
    let mut session_doc = serde_json::json!({"id":session});
    if report.session_established {
        session_doc["resume_id"] = session.into();
    }
    let mut model = serde_json::Map::new();
    if let Some(id) = &profile.model {
        model.insert("id".into(), id.clone().into());
    }
    if !profile.options.is_empty() {
        model.insert("options".into(), serde_json::json!(profile.options));
    }
    let decision_timeout_ms = crate::native_guard::decision_wait()
        .saturating_mul(1000)
        .saturating_add(10_000);
    let document = serde_json::json!({
        "schema":"acio.input/1",
        "request_id":format!("{session}-{}", report.attempt),
        "mode":"task",
        "role":"worker",
        "instruction":{"text":prompt},
        "task":{"type":task.task_type.unwrap_or(crate::plan::TaskType::Code),"completion":{"criteria":[task.prompt],"verify":[task.test_command]}},
        "workspace":{"root":cwd.to_str().ok_or("non-UTF8 task workspace")?,"access":"workspace_write","paths":task.paths},
        "session":session_doc,
        "model":model,
        "limits":{"timeout_ms":timeout.saturating_mul(1000).max(1000),"max_event_bytes":1024*1024},
        "control":{"enabled":true,"decision_timeout_ms":decision_timeout_ms}
    });
    crate::cli_io::validate("input", &document)
        .map_err(|errors| format!("invalid ACIO worker input: {}", errors.join("; ")))?;
    Ok(crate::acio::input_line(&document))
}

pub(super) async fn run(
    spec: crate::process::ProcessSpec,
    task: &Task,
    report: &TaskReport,
    profile: &AgentProfile,
) -> Result<Outcome, String> {
    let pending = Arc::new(Mutex::new(None));
    let paused = Arc::new(AtomicBool::new(false));
    let dir = report.artifacts.parent().ok_or("run directory missing")?;
    let stop = dir.join("stop.request");
    let spool = report.artifacts.join("process/stdout.jsonl");
    let database = dir.join("state.sqlite");
    let (run_dir, task_id, session, stop) = (
        dir.to_path_buf(),
        task.id.clone(),
        report.session_id.clone().ok_or("owned session missing")?,
        stop,
    );
    let stop_path = stop.clone();
    let waiting = pending.clone();
    let deciding = paused.clone();
    let decide = move |request: crate::acio::ToolRequest| {
        let (waiting, deciding, run_dir, task_id, session, stop) = (
            waiting.clone(),
            deciding.clone(),
            run_dir.clone(),
            task_id.clone(),
            session.clone(),
            stop.clone(),
        );
        async move {
            let gate_request = crate::gates::ToolRequest {
                tool: format!("acio:{}", request.name),
                input: serde_json::json!({"call_id":request.call_id,"input":request.input}),
                reason: format!(
                    "ACIO tool {} requests an exact human decision",
                    request.name
                ),
            };
            *waiting
                .lock()
                .map_err(|_| "pending ACIO request poisoned")? = Some(gate_request.clone());
            deciding.store(true, Ordering::Release);
            let result = crate::native_guard::decide_acio_tool(
                run_dir,
                task_id,
                gate_request,
                session,
                &stop,
            )
            .await;
            deciding.store(false, Ordering::Release);
            match result {
                Ok(Some(decision)) => {
                    waiting
                        .lock()
                        .map_err(|_| "pending ACIO request poisoned")?
                        .take();
                    Ok(if decision.allow {
                        crate::acio::Decision::Allow
                    } else {
                        crate::acio::Decision::Deny
                    })
                }
                Ok(None) => Ok(crate::acio::Decision::Deny),
                Err(error) => {
                    waiting
                        .lock()
                        .map_err(|_| "pending ACIO request poisoned")?
                        .take();
                    Err(error)
                }
            }
        }
    };
    let observed = crate::progress::observe_profile(
        crate::acio::run(
            spec,
            decide,
            crate::acio::Options {
                stop_file: Some(stop_path),
                paused: Some(paused),
                ..Default::default()
            },
        ),
        &spool,
        &database,
        &task.id,
        report.attempt as u64,
        profile,
    )
    .await?;
    Ok(Outcome {
        pending: pending
            .lock()
            .map_err(|_| "pending ACIO request poisoned")?
            .clone(),
        process: observed.map(|run| run.result),
    })
}
