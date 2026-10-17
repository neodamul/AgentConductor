use super::*;

/// The CLI's identifier of the tool call a pre-tool callback is about.
pub(super) fn tool_use_id(message: &serde_json::Value) -> Result<&str, String> {
    message["request"]["tool_use_id"]
        .as_str()
        .or_else(|| message["request"]["input"]["tool_use_id"].as_str())
        .filter(|id| !id.is_empty() && id.len() <= 256)
        .ok_or_else(|| "native tool-use identity missing".to_string())
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Holds the pre-tool callback open until a human decides this exact request.
/// Returns None when no decision arrives, so the caller falls back to stopping.
pub(super) async fn live_decision(
    binding: &Binding,
    request: &ToolRequest,
    call: &str,
    session: &str,
    stop: &std::path::Path,
) -> Result<Option<ToolDecision>, String> {
    let wait = decision_wait();
    if wait == 0 {
        return Ok(None);
    }
    // Bind the decision to this tool call: an identical command in a later call (or a
    // resumed attempt) must not reuse an earlier consumed or approved ledger entry.
    let request = &ToolRequest {
        reason: format!("{} (tool call {call})", request.reason),
        ..request.clone()
    };
    let engine = std::env::current_exe().map_err(|e| e.to_string())?;
    let db = binding.dir.join("state.sqlite");
    let context =
        crate::workflow::verify_tool_context(&binding.dir, &binding.task, request, &engine).await?;
    let started = unix_now();
    let id = crate::gates::Ledger::open(&db)?.request(&context, request, started)?;
    crate::state::Store::open(&db)?.record(
        "tool_live_wait",
        &serde_json::json!({"id":id,"task":binding.task}),
        "Worker paused before execution for an exact human tool decision",
    )?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(wait);
    while std::time::Instant::now() < deadline {
        if stop.exists() {
            return Err("Human stop while awaiting a tool decision".into());
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        // Newest first; a decision may re-key the request to the current workspace state.
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
                let fresh = crate::workflow::verify_tool_context(
                    &binding.dir,
                    &binding.task,
                    request,
                    &engine,
                )
                .await?;
                // The worker stayed paused since the decision; parallel owned-scope writes
                // may change the shared workspace, but every other binding must still match.
                let bound = crate::gates::Context {
                    workspace_sha256: latest.context.workspace_sha256.clone(),
                    ..fresh.clone()
                };
                let mut ledger = crate::gates::Ledger::open(&db)?;
                if ledger.consume(&latest.id, &bound, request, unix_now())?
                    == crate::gates::Decision::Allowed
                {
                    return Ok(Some(ToolDecision {
                        allow: true,
                        reason: "Exact live human approval consumed once".into(),
                    }));
                }
                // The workspace changed after the decision; ask again for the current state.
                ledger.supersede(&latest.id)?;
                ledger.request(&fresh, request, unix_now())?;
            }
            _ => {}
        }
    }
    Ok(None)
}

/// Operating-system CSPRNG bytes (DD-37).
pub(super) fn random_bytes(bytes: &mut [u8]) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::io::Read;
        std::fs::File::open("/dev/urandom")
            .and_then(|mut file| file.read_exact(bytes))
            .map_err(|e| e.to_string())
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::Security::Cryptography::{
            BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom,
        };
        let length = u32::try_from(bytes.len()).map_err(|_| "random request too large")?;
        let status = unsafe {
            BCryptGenRandom(
                std::ptr::null_mut(),
                bytes.as_mut_ptr(),
                length,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        };
        if status < 0 {
            return Err(format!("system random generator failed: {status:#x}"));
        }
        Ok(())
    }
}
