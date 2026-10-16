use super::{Report, availability_timer, digest, now};
use crate::{git::Repository, state::Store};
use std::{fs, io::Write, path::Path};

pub(super) fn save(store: &mut Store, dir: &Path, report: &Report) -> Result<(), String> {
    // JSON references do not protect Git objects from garbage collection.
    let git = Repository::open(&report.repo)?;
    let run_key = digest(dir.to_string_lossy().as_bytes());
    for snapshot in report
        .tasks
        .iter()
        .filter_map(|t| t.snapshot.as_ref())
        .chain(report.candidate.iter())
    {
        git.command(&[
            "update-ref",
            &format!("refs/codex/agentconductor/{run_key}/{}", snapshot.commit),
            &snapshot.commit,
        ])?;
    }
    let value = serde_json::to_value(report).map_err(|e| e.to_string())?;
    let summary = format!(
        "AgentConductor run={} goal={} phase={} completed={} failed={}",
        dir.display(),
        report.plan.goal,
        report.state,
        report
            .tasks
            .iter()
            .filter(|t| t.status == "completed")
            .count(),
        report.tasks.iter().filter(|t| t.status == "failed").count()
    );
    store.record("workflow", &value, &summary)?;
    availability_timer::record_timer(store, dir, report, (now() / 1000) as i64)?;
    fs::write(
        dir.join("report.json"),
        serde_json::to_vec_pretty(report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    // DD-79: a linked run reports notifiable state changes; delivery never fails the run.
    let done = report
        .tasks
        .iter()
        .filter(|t| t.status == "completed")
        .count();
    crate::messenger::notify_run(dir, &report.state, done, report.tasks.len());
    let wiki = dir.join("knowledge");
    fs::create_dir_all(&wiki).map_err(|e| e.to_string())?;
    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(wiki.join("log.md"))
        .map_err(|e| e.to_string())?;
    writeln!(log, "\n- observed_at_ms={} {}", now(), summary).map_err(|e| e.to_string())?;
    fs::write(wiki.join("index.md"),"# Run knowledge\n\n- [Progress log](log.md)\n- [Draft lessons](lessons.md)\n- [Reviewed runtime lessons](wiki/index.md)\n\nSQLite is authoritative; these pages are local progress summaries. Reviewed lesson links are populated after exact promotion.\n").map_err(|e|e.to_string())?;
    if !wiki.join("lessons.md").exists() {
        fs::write(
            wiki.join("lessons.md"),
            "# Lessons\n\nRecord context, correction, evidence, confidence, and scope here.\n",
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
