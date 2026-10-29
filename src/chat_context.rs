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
