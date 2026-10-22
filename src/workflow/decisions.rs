//! DD-177: every judgment the coordinator makes, derived from the audit records it already
//! writes. There is no second log and no setting that hides a row.
use super::*;
use serde_json::{Value, json};

/// The audit record kinds that hold a coordinator judgment.
pub const EVENTS: &[&str] = &[
    "tool_orchestrator_decision",
    "coordinator_review",
    "test_command_revised",
    "task_timeout_resume",
    "auto_recovery",
    "handoff",
    "step_lessons",
];
const LIMIT: usize = 2000;

fn text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn row(record: &Value) -> Option<Value> {
    let p = &record["payload"];
    let (kind, judgment, task, detail, applied) = match record["event"].as_str()? {
        "tool_orchestrator_decision" => (
            "tool",
            match p["allow"] {
                Value::Bool(true) => "allow",
                Value::Bool(false) => "deny",
                _ => "undecided",
            }
            .to_owned(),
            text(&p["task"]),
            format!(
                "{}: {}",
                text(&p["tool"]),
                if p["reason"].is_null() {
                    text(&p["error"])
                } else {
                    text(&p["reason"])
                }
            ),
            Value::Null,
        ),
        "coordinator_review" => {
            let revisions = p["proposal"]["revisions"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            let mut detail = text(&p["proposal"]["summary"]);
            for r in &revisions {
                detail.push_str(&format!("\n{}: {}", text(&r["task"]), text(&r["feedback"])));
            }
            (
                "review",
                text(&p["proposal"]["verdict"]),
                revisions
                    .first()
                    .map(|r| text(&r["task"]))
                    .unwrap_or_default(),
                detail,
                p["apply"].clone(),
            )
        }
        "test_command_revised" => (
            "test_command",
            "replaced".to_owned(),
            text(&p["task"]),
            format!("{} -> {}: {}", p["from"], p["to"], text(&p["reason"])),
            Value::Null,
        ),
        "task_timeout_resume" => (
            "timeout",
            "resume".to_owned(),
            text(&p["task"]),
            format!(
                "attempt {} reached its time limit; resumed automatically",
                text(&p["attempt"])
            ),
            Value::Null,
        ),
        "auto_recovery" => (
            "recovery",
            text(&p["status"]),
            String::new(),
            format!("attempts {}", text(&p["attempts"])),
            Value::Null,
        ),
        "handoff" => (
            "handoff",
            "reassigned".to_owned(),
            text(&p["task"]),
            format!("{} -> {}", text(&p["from"]), text(&p["to"])),
            Value::Null,
        ),
        "step_lessons" => {
            let newest = p.as_array().and_then(|a| a.first())?;
            (
                "lesson",
                "recorded".to_owned(),
                text(&newest["task"]),
                format!(
                    "{} => {}",
                    text(&newest["cause"]),
                    text(&newest["response"])
                ),
                Value::Null,
            )
        }
        _ => return None,
    };
    Some(
        json!({"seq":record["seq"],"kind":kind,"judgment":judgment,"task":task,"detail":detail,
        "applied":applied,"summary":record["summary"]}),
    )
}

/// Every judgment of the run, oldest first.
pub fn list(dir: &Path) -> Result<Vec<Value>, String> {
    let store = Store::open(dir.join("state.sqlite"))?;
    Ok(store
        .audit_of(EVENTS, LIMIT)?
        .iter()
        .filter_map(row)
        .collect())
}

pub fn count(dir: &Path) -> Result<usize, String> {
    Store::open(dir.join("state.sqlite"))?.count_events(EVENTS)
}
