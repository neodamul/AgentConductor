//! GitHub #1009: what a run notice says beyond its event — the Phase, each task by name and
//! state, and, when the run cannot proceed, the error and the blocking cause.
use crate::workflow::Report;

/// Position of the run's Phase in its Goal: (1-based index, total, name).
pub type PhasePosition = (usize, usize, String);

const MAX_TASKS: usize = 12;
const MAX_ERROR: usize = 300;

fn clip(text: &str, limit: usize) -> String {
    let line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.chars().count() > limit {
        line.chars().take(limit).collect::<String>() + "…"
    } else {
        line
    }
}

fn status(state: &str, korean: bool) -> &str {
    match (state, korean) {
        ("completed", true) => "완료",
        ("running", true) => "실행 중",
        ("pending", true) => "대기",
        ("failed", true) => "실패",
        ("stopped", true) => "중지",
        ("waiting_availability", true) => "사용량 대기",
        ("waiting_tool_approval", true) => "도구 승인 대기",
        ("blocked", true) => "차단됨",
        ("waiting_availability", false) => "waiting for usage",
        ("waiting_tool_approval", false) => "waiting for tool approval",
        (other, _) => other,
    }
}

/// GitHub #1028: the first content line, skipping label lines (`Goal:`) and bare tags, as the
/// Room's `taskLine()` does.
fn task_line(prompt: &str) -> Option<&str> {
    prompt
        .lines()
        .map(|l| {
            let l = l.trim();
            l.strip_prefix("- ")
                .or_else(|| l.strip_prefix("* "))
                .map_or(l, str::trim_start)
        })
        .find(|l| {
            let name = l
                .strip_prefix('<')
                .and_then(|l| l.strip_suffix('>'))
                .map(|l| l.trim_start_matches('/'));
            let tag = name.is_some_and(|n| {
                !n.is_empty() && n.chars().all(|c| c.is_alphanumeric() || "_-".contains(c))
            });
            !l.is_empty() && !l.ends_with(':') && !tag
        })
}

/// The lines below the notice headline, already localized.
pub fn run_detail(report: &Report, phase: Option<&PhasePosition>, korean: bool) -> String {
    let mut lines = vec![];
    if let Some((index, total, name)) = phase {
        lines.push(if name.trim().is_empty() {
            format!("Phase {index}/{total}")
        } else {
            format!("Phase {index}/{total} · {}", clip(name, 60))
        });
    }
    let done = report
        .tasks
        .iter()
        .filter(|t| t.status == "completed")
        .count();
    lines.push(if korean {
        format!("작업 {done}/{} 완료", report.tasks.len())
    } else {
        format!("Tasks {done}/{} completed", report.tasks.len())
    });
    for task in report.tasks.iter().take(MAX_TASKS) {
        let summary = report
            .plan
            .tasks
            .iter()
            .find(|p| p.id == task.id)
            .and_then(|p| task_line(&p.prompt))
            .map(|l| format!(" {}", clip(l, 60)))
            .unwrap_or_default();
        lines.push(format!(
            "· {}{summary} — {}",
            task.id,
            status(&task.status, korean)
        ));
    }
    if report.tasks.len() > MAX_TASKS {
        lines.push(format!("· … +{}", report.tasks.len() - MAX_TASKS));
    }
    // The run cannot proceed: say what failed and what blocks it.
    if matches!(report.state.as_str(), "failed" | "blocked" | "stopped") {
        if let Some(error) = report.error.as_deref().filter(|e| !e.trim().is_empty()) {
            lines.push(format!(
                "{}: {}",
                if korean { "오류" } else { "Error" },
                clip(error, MAX_ERROR)
            ));
        }
        let mut causes: Vec<String> = report
            .tasks
            .iter()
            .filter(|t| t.status == "failed" || t.status == "stopped")
            .filter_map(|t| {
                t.error
                    .as_deref()
                    .filter(|e| !e.trim().is_empty())
                    .map(|e| format!("{} — {}", t.id, clip(e, MAX_ERROR)))
            })
            .collect();
        if let Some(review) = report
            .review_error
            .as_deref()
            .filter(|e| !e.trim().is_empty())
        {
            causes.push(clip(review, MAX_ERROR));
        }
        for cause in causes.into_iter().take(4) {
            lines.push(format!(
                "{}: {cause}",
                if korean {
                    "차단 원인"
                } else {
                    "Blocked by"
                }
            ));
        }
    }
    lines.join("\n")
}
