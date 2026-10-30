//! GitHub #243 (DD-200): what the chat coordinator is told about the workspace and the latest run. It reads only
//! the project folder, so the run's evidence (plan, results, diff) is quoted into its prompt instead.
use crate::workflow::Report;
use serde_json::Value;
use std::{fmt::Write, path::Path};

const MAX_DIFF_BYTES: usize = 12 * 1024;
const MAX_LIST: usize = 40;
const PROMPT_LINE: usize = 200;

fn cut(text: &str, max: usize) -> &str {
    let mut end = text.len().min(max);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// The section for the saved workspace settings (`startup.json`); the run's report and diff are read when present.
pub fn workspace(saved: &Value, project: &Path) -> String {
    let run = saved["last_run"]
        .as_str()
        .map(std::path::PathBuf::from)
        .and_then(|dir| {
            let report = crate::workflow::load(&dir).ok()?;
            let diff = std::fs::read(dir.join("diff.patch")).unwrap_or_default();
            Some((report, String::from_utf8_lossy(&diff).into_owned()))
        });
    render(
        &saved["config"],
        project,
        run.as_ref().map(|(report, diff)| (report, diff.as_str())),
    )
}

/// Markdown for the prompt's Document; every part is bounded and the result stays under about 20 KiB.
pub fn render(config: &Value, project: &Path, run: Option<(&Report, &str)>) -> String {
    let text = |key: &str| config[key].as_str().unwrap_or_default().trim();
    let mut out = String::from("## Workspace\n\n");
    let _ = writeln!(out, "Project folder: {}", project.display());
    let _ = writeln!(out, "Workspace mode: {}", text("workspace_mode"));
    let _ = writeln!(out, "Goal: {}\n", cut(text("goal"), 2000));
    let Some((report, diff)) = run else {
        out.push_str("No run has started yet, so there are no results to describe.\n");
        return crate::prompt_form::neutral(&out);
    };
    let _ = writeln!(out, "Latest run: {}", report.state);
    if let Some(error) = &report.error {
        let _ = writeln!(out, "Run error: {}", cut(error, 500));
    }
    out.push_str("\nTasks:\n");
    for task in report.tasks.iter().take(MAX_LIST) {
        let plan = report.plan.tasks.iter().find(|t| t.id == task.id);
        let _ = write!(out, "- {} [{}]", task.id, task.status);
        if let Some(plan) = plan {
            let prompt = plan.prompt.lines().next().unwrap_or_default();
            let _ = write!(
                out,
                " agent {}, owned paths {}: {}",
                plan.agent,
                plan.paths.join(", "),
                cut(prompt, PROMPT_LINE)
            );
        }
        if let Some(error) = &task.error {
            let _ = write!(out, " (error: {})", cut(error, 300));
        }
        out.push('\n');
    }
    out.push_str("\nTests:\n");
    for test in report.tests.iter().take(MAX_LIST) {
        let result = match (test.exit_code, test.timed_out) {
            (_, true) => "timed out".to_string(),
            (Some(code), _) => format!("exit {code}"),
            _ => "no exit code".to_string(),
        };
        let _ = writeln!(out, "- {}: {result}", test.task);
    }
    if let Some(candidate) = &report.candidate {
        let _ = writeln!(
            out,
            "\nFiles changed by the candidate: {}",
            candidate
                .changed
                .iter()
                .take(MAX_LIST)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if !diff.trim().is_empty() {
        let shown = cut(diff, MAX_DIFF_BYTES);
        let _ = write!(out, "\nCandidate diff:\n\n{shown}");
        if !shown.ends_with('\n') {
            out.push('\n');
        }
        if shown.len() < diff.len() {
            let _ = writeln!(
                out,
                "(diff cut after {} of {} bytes)",
                shown.len(),
                diff.len()
            );
        }
    }
    crate::prompt_form::neutral(&out)
}

const MAX_TAIL: usize = 1500;

/// The end of a log, as text, starting at a line boundary when the log was cut.
fn tail(path: &Path, max: usize) -> String {
    let Ok(bytes) = std::fs::read(path) else {
        return String::new();
    };
    let cut = bytes.len() > max;
    let text = String::from_utf8_lossy(&bytes[bytes.len().saturating_sub(max)..]).into_owned();
    let text = match text.split_once('\n') {
        Some((_, rest)) if cut => rest,
        _ => &text,
    };
    text.trim().to_owned()
}

/// GitHub #261 (DD-204): which plan or start job is running or has finished, for how long, and how much output its
/// process has written and when it last wrote, so the coordinator can say why a plan takes long. It only reads files.
pub fn running(saved: &Value, root: &Path, now_ms: u64) -> String {
    let mut out = String::from("## Running work\n\n");
    let Some(id) = saved["job"]["id"]
        .as_str()
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
    else {
        out.push_str("Nothing is running: no plan or start job is active.\n");
        return crate::prompt_form::neutral(&out);
    };
    let kind = saved["job"]["operation"].as_str().unwrap_or("job");
    let owner: Value = std::fs::read(root.join(format!("coordinator-ui-{id}/process/owner.json")))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or(Value::Null);
    let state = if owner["phase"] == "terminal" {
        format!("finished with exit code {}", owner["exit_code"])
    } else {
        "running".into()
    };
    let started = saved["job"]["started_ms"].as_u64().filter(|ms| *ms > 0);
    let _ = write!(out, "The {kind} job is {state}");
    if let Some(started) = started {
        let _ = write!(
            out,
            "; it started {} s ago",
            now_ms.saturating_sub(started) / 1000
        );
    }
    out.push_str(".\n");
    if let Some(phase) = saved["proposal"]["slug_phase"].as_str() {
        let _ = writeln!(out, "Planning phase: {}", cut(phase, PROMPT_LINE));
    }
    // The planner's own output is in the proposal folder; any other job writes into its process folder.
    let (dir, output) = match (kind, saved["proposal"]["directory"].as_str()) {
        ("plan", Some(proposal)) => (Path::new(proposal).join("process"), "stdout.jsonl"),
        _ => (
            root.join(format!("coordinator-ui-{id}/process")),
            "stdout.log",
        ),
    };
    let (mut bytes, mut last) = (0u64, 0u64);
    for name in [output, "stderr.log"] {
        if let Ok(metadata) = std::fs::metadata(dir.join(name)) {
            bytes += metadata.len();
            let ms = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_millis() as u64);
            last = last.max(ms);
        }
    }
    if last == 0 {
        out.push_str("Its process has written no output yet.\n");
    } else {
        let _ = writeln!(
            out,
            "Its process has written {bytes} bytes in total; the last write was {} s ago.",
            now_ms.saturating_sub(last) / 1000
        );
    }
    for (title, name) in [
        ("Latest output (end)", output),
        ("Latest errors (end)", "stderr.log"),
    ] {
        let text = tail(&dir.join(name), MAX_TAIL);
        if !text.is_empty() {
            let _ = write!(out, "\n{title}:\n\n{text}\n");
        }
    }
    crate::prompt_form::neutral(&out)
}
