//! GitHub #331: why an update cannot start, in terms of the work that blocks it.
use crate::ui::messages::{in_language, interpolate};
use std::path::Path;

/// Work that would end with the app, or a check that could not be made.
#[derive(Debug)]
pub enum Blocker {
    Active {
        workspace: String,
        what: String,
        elapsed_ms: Option<u64>,
    },
    Unknown {
        what: String,
        error: String,
    },
}

fn clock(ms: u64) -> String {
    format!("{}:{:02}", ms / 60_000, ms / 1000 % 60)
}

/// The refusal text: running work (workspace, work, time, how to get out) apart from checks that failed.
pub fn message(language: &str, blockers: &[Blocker]) -> String {
    let text = |id| in_language(language, id);
    let mut running = Vec::new();
    let mut unknown = Vec::new();
    for blocker in blockers {
        match blocker {
            Blocker::Active {
                workspace,
                what,
                elapsed_ms,
            } => {
                let elapsed = elapsed_ms.map_or(String::new(), |ms| {
                    interpolate(text("native.update_elapsed"), &[("time", &clock(ms))])
                });
                running.push(interpolate(
                    text("native.update_blocker_line"),
                    &[
                        ("workspace", workspace),
                        ("what", what),
                        ("elapsed", &elapsed),
                    ],
                ));
            }
            Blocker::Unknown { what, error } => unknown.push(interpolate(
                text("native.update_check_line"),
                &[("what", what), ("error", error)],
            )),
        }
    }
    let mut parts = Vec::new();
    if !running.is_empty() {
        parts.push(text("native.update_blocked_active").to_string());
        parts.extend(running);
        parts.push(text("native.update_blocked_advice").to_string());
    }
    if !unknown.is_empty() {
        parts.push(text("native.update_check_failed").to_string());
        parts.extend(unknown);
        parts.push(text("native.update_check_advice").to_string());
    }
    parts.join("\n")
}

/// A job kind (`quota:claude`, `plan`, ...) as the work it is.
pub fn job_label(language: &str, kind: &str) -> String {
    let text = |id| in_language(language, id);
    let (head, agent) = kind.split_once(':').unwrap_or((kind, ""));
    let label = match head {
        "plan" => text("native.update_job_plan"),
        "start" => text("native.update_job_start"),
        "quota" => text("native.update_job_quota"),
        "catalog" => text("native.update_job_catalog"),
        "chat" => text("native.update_job_chat"),
        _ => text("native.update_job_other"),
    };
    if agent.is_empty() {
        label.into()
    } else {
        format!("{label} ({agent})")
    }
}

/// A run blocks an update only while its coordinator holds the run lock; a report that still says
/// "running" with nobody holding the lock is a run that ended without saying so.
pub fn run_blocker(language: &str, workspace: &str, run: &Path) -> Option<Blocker> {
    let report = crate::workflow::load(run).ok()?;
    if report.state != "running" && !report.tasks.iter().any(|t| t.status == "running") {
        return None;
    }
    let what = in_language(language, "native.update_job_run").to_string();
    let elapsed_ms = std::fs::metadata(run)
        .and_then(|m| m.created())
        .ok()
        .and_then(|created| created.elapsed().ok())
        .map(|age| age.as_millis() as u64);
    match crate::leases::Lease::acquire(&run.join("coordinator.lock")) {
        Ok(_free) => None,
        Err(e) if e.starts_with("lease is held") => Some(Blocker::Active {
            workspace: workspace.into(),
            what,
            elapsed_ms,
        }),
        Err(e) => Some(Blocker::Unknown {
            what: format!("{workspace}: {what} {}", run.display()),
            error: e,
        }),
    }
}
