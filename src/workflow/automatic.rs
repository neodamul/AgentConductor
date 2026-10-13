//! DD-128: bounded recovery of interrupted coordinators; never touch a live owner.
use super::*;
use serde_json::{Value, json};

#[derive(Clone, Debug, Default, Serialize)]
pub struct AutomaticRecovery {
    pub recovered: bool,
    pub resume: bool,
    pub attempts: u32,
    pub reason: Option<String>,
}

fn progress(report: &Report) -> (usize, u128) {
    let completed: Vec<_> = report
        .tasks
        .iter()
        .filter(|t| t.status == "completed")
        .collect();
    (
        completed.len(),
        completed
            .iter()
            .filter_map(|t| t.finished_ms)
            .max()
            .unwrap_or(0),
    )
}

pub fn automatic_recover(dir: &Path, enabled: bool) -> Result<AutomaticRecovery, String> {
    let _lease = match crate::leases::Lease::acquire(&dir.join("auto-recovery.lock")) {
        Ok(lease) => lease,
        Err(_) => return Ok(AutomaticRecovery::default()),
    };
    let mut report = load(dir)?;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    let prior = store
        .optional_document("auto_recovery")?
        .unwrap_or(Value::Null);
    let recovered = if report.state == "running" {
        match super::recovery::recover_interrupted(dir, true) {
            Ok(recovered) => {
                report = recovered;
                true
            }
            Err(reason) => {
                return Ok(AutomaticRecovery {
                    reason: Some(reason),
                    ..Default::default()
                });
            }
        }
    } else {
        // Only a durable, unchanged recovery claim may restart a stopped run.
        if report.state != "stopped"
            || !matches!(
                prior["status"].as_str(),
                Some("resume_pending" | "resume_failed")
            )
            || prior["revision"].as_str() != Some(revision(&report)?.as_str())
        {
            return Ok(AutomaticRecovery::default());
        }
        false
    };
    let (done, finished_ms) = progress(&report);
    let progressed = done > prior["completed"].as_u64().unwrap_or(0) as usize
        || finished_ms > prior["completed_ms"].as_u64().unwrap_or(0) as u128;
    let previous = if progressed {
        0
    } else {
        prior["attempts"].as_u64().unwrap_or(0).min(2) as u32
    };
    let explicitly_stopped = stop_digest(dir)? != "absent";
    let resume = enabled && !explicitly_stopped && previous < 2;
    let attempts = previous + u32::from(resume);
    let status = if explicitly_stopped {
        "user_stopped"
    } else if !enabled {
        "recovered"
    } else if !resume {
        "retry_limit"
    } else {
        "resume_pending"
    };
    store.record("auto_recovery", &json!({"status":status,"recovered":recovered,"attempts":attempts,"completed":done,"completed_ms":finished_ms,"revision":revision(&report)?,"observed_at_ms":now()}),
        "Automatic recovery verified process absence; continuation remains bounded and approvals revoked")?;
    Ok(AutomaticRecovery {
        recovered,
        resume,
        attempts,
        reason: None,
    })
}

pub fn automatic_resume_result(
    dir: &Path,
    started: bool,
    reason: Option<&str>,
) -> Result<(), String> {
    let mut store = Store::open(dir.join("state.sqlite"))?;
    let Some(mut receipt) = store.optional_document("auto_recovery")? else {
        return Ok(());
    };
    receipt["status"] = if started { "resumed" } else { "resume_failed" }.into();
    receipt["reason"] = json!(reason);
    store.record(
        "auto_recovery",
        &receipt,
        if started {
            "Automatically resumed an interrupted run"
        } else {
            "Automatic resume could not launch; manual review is available"
        },
    )
}

pub fn auto_resume_enabled(dir: &Path) -> bool {
    let path = dir.with_extension("auto-resume.json");
    std::fs::metadata(&path)
        .ok()
        .filter(|m| m.is_file() && m.len() <= 1024)
        .and_then(|_| std::fs::read(&path).ok())
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .and_then(|v| v["auto_resume"].as_bool())
        .unwrap_or(true)
}

pub fn set_auto_resume(dir: &Path, enabled: bool) -> Result<(), String> {
    // A sidecar is written before the worker creates the fresh run directory.
    let path = dir.with_extension("auto-resume.json");
    let temporary = dir.with_extension(format!(
        "auto-resume-{}.pending",
        crate::native_guard::session_id()?
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    file.write_all(
        &serde_json::to_vec(&json!({"auto_resume":enabled})).map_err(|e| e.to_string())?,
    )
    .and_then(|_| file.sync_all())
    .map_err(|e| e.to_string())?;
    fs::rename(&temporary, path).map_err(|e| e.to_string())
}
