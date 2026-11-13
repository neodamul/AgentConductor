//! GitHub #828: each agent's task sessions across this Workspace's runs, with stop and clean-up.
use super::*;

fn finished_run(state: &str) -> bool {
    matches!(state, "completed" | "stopped" | "failed")
}

fn session_state(status: &str, run_state: &str) -> &'static str {
    match status {
        // A task still marked running in a finished run has no coordinator left to drive it.
        "running" if finished_run(run_state) => "stale",
        "running" => "running",
        "completed" | "approved" | "denied" => "done",
        "failed" => "failed",
        "stopped" => "stopped",
        _ => "waiting",
    }
}

// #955: reuse the loaded report; do not scan every saved session on every snapshot.
pub(super) fn session_revision(run: &Path, report: &workflow::Report) -> String {
    let tasks: Vec<_> = report
        .tasks
        .iter()
        .map(|task| {
            let agent = report
                .plan
                .tasks
                .iter()
                .find(|planned| planned.id == task.id)
                .map(|planned| &planned.agent);
            serde_json::json!([
                task.id,
                task.status,
                task.started_ms,
                task.finished_ms,
                agent
            ])
        })
        .collect();
    workflow::digest(
        serde_json::json!([run, report.state, tasks])
            .to_string()
            .as_bytes(),
    )
}

impl Startup {
    fn session_run(&self, run: &str) -> Result<PathBuf, String> {
        if !run.starts_with("run-") || run.contains(['/', '\\']) || run.contains("..") {
            return Err("unknown session run".into());
        }
        let dir = self.broker.dir.join(run);
        if !dir.join("report.json").is_file() {
            return Err("unknown session run".into());
        }
        Ok(dir)
    }

    pub(super) fn agent_sessions(&self) -> Result<serde_json::Value, String> {
        let mut agents = serde_json::Map::new();
        let mut rows = vec![];
        for item in fs::read_dir(&self.broker.dir).map_err(|e| e.to_string())? {
            let item = item.map_err(|e| e.to_string())?;
            let run = item.file_name().to_string_lossy().to_string();
            if !run.starts_with("run-") || !item.path().join("report.json").is_file() {
                continue;
            }
            // A run that cannot be read (for example while it is being created) is skipped, not fatal.
            let Ok(report) = workflow::load(&item.path()) else {
                continue;
            };
            for (index, task) in report.tasks.iter().enumerate() {
                let Some(started) = task.started_ms else {
                    continue;
                };
                if task.artifacts.join(workflow::SESSION_CLEANED).exists() {
                    continue;
                }
                let Some(planned) = report.plan.tasks.iter().find(|p| p.id == task.id) else {
                    continue;
                };
                let state = session_state(&task.status, &report.state);
                let line = planned
                    .prompt
                    .lines()
                    .map(str::trim)
                    .find(|l| !l.is_empty())
                    .unwrap_or(&task.id);
                rows.push((
                    started,
                    planned.agent.clone(),
                    serde_json::json!({
                        "run": run,
                        "task": task.id,
                        "label": format!("T{} {}", index + 1, line.chars().take(120).collect::<String>()),
                        "goal": report.plan.goal.lines().next().unwrap_or_default(),
                        "state": state,
                        "started_ms": started,
                        "last_ms": task.finished_ms.unwrap_or(started),
                        "note": task.error.as_deref().and_then(|e| e.lines().next()).map(|e| e.chars().take(120).collect::<String>()),
                        "cleanable": finished_run(&report.state) && !matches!(state, "running" | "waiting"),
                        "stoppable": matches!(state, "running" | "waiting") && !finished_run(&report.state),
                    }),
                ));
            }
        }
        rows.sort_by_key(|row| std::cmp::Reverse(row.0));
        for (_, agent, row) in rows {
            agents
                .entry(agent)
                .or_insert_with(|| serde_json::json!([]))
                .as_array_mut()
                .unwrap()
                .push(row);
        }
        Ok(serde_json::Value::Object(agents))
    }

    pub(super) fn stop_session(&self, run: &str) -> Result<serde_json::Value, String> {
        workflow::stop(
            &self.session_run(run)?,
            "Stopped from the agent sessions list",
        )?;
        self.agent_sessions()
    }

    pub(super) fn clean_sessions(
        &self,
        sessions: &[(String, String)],
    ) -> Result<serde_json::Value, String> {
        for (run, task) in sessions {
            workflow::clean_session(&self.session_run(run)?, task)?;
        }
        self.agent_sessions()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/agent_sessions.rs"]
mod session_tests;
