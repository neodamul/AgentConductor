use super::*;

impl Startup {
    pub(in crate::ui::setup) fn delete_goal(
        &self,
        saved: &mut Saved,
        slug: &str,
        goal: &str,
        confirmed_root: &Path,
    ) -> Result<serde_json::Value, String> {
        let root = self.goal_root(saved, slug)?;
        if self.goal_entry(saved, slug)?.goal != goal || root != confirmed_root {
            return Err("Goal or folder changed; review the permanent deletion again".into());
        }
        let audit = serde_json::json!({"slug":slug,"goal":goal,"root":root});
        crate::state::Store::open(self.broker.dir.join("state.sqlite"))?.record(
            "goal_delete_requested",
            &audit,
            "Human confirmed permanent Goal deletion; no backup",
        )?;
        if saved
            .proposal
            .as_ref()
            .is_some_and(|p| p.slug.as_deref() == Some(slug))
            && self.job(saved)?["state"] == "running"
        {
            if saved.job.as_ref().is_some_and(|j| j.operation == "plan") {
                self.stop_planning(saved)?;
            } else if let Some(run) = self.selected_goal_run(saved, slug)? {
                request_stop(&run)?;
            }
            return Ok(serde_json::json!({"deleted":false,"stopping":true}));
        }
        let mut runs = std::collections::BTreeSet::new();
        if let Some(run) = self.selected_goal_run(saved, slug)? {
            runs.insert(run);
        }
        // Earlier phases may still have a coordinator; match their recorded folder, not Goal text.
        for entry in fs::read_dir(&self.broker.dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if !entry.file_name().to_string_lossy().starts_with("run-")
                || !entry.path().join("report.json").exists()
            {
                continue;
            }
            let path = entry.path();
            let link = path.join("slug-phase.json");
            if link.exists() {
                let value: serde_json::Value = read(&link)?;
                if value["root"].as_str().map(Path::new) == Some(root.as_path()) {
                    runs.insert(self.verified_goal_run(saved, slug, &path)?);
                }
            }
        }
        let mut locks = Vec::new();
        let mut stopping = false;
        for run in &runs {
            let report = workflow::load(run)?;
            if !matches!(
                report.state.as_str(),
                "completed"
                    | "failed"
                    | "stopped"
                    | "rejected"
                    | "blocked"
                    | "waiting_approval"
                    | "approved"
            ) || report.tasks.iter().any(|t| t.status == "running")
            {
                request_stop(run)?;
                stopping = true;
                continue;
            }
            match crate::leases::Lease::acquire(&run.join("coordinator.lock")) {
                Ok(lock) => locks.push(lock),
                Err(_) => {
                    stopping = true;
                    continue;
                }
            }
            verify_processes(run, &mut 20_000)?;
        }
        if stopping {
            return Ok(serde_json::json!({"deleted":false,"stopping":true}));
        }
        self.ensure_no_live_jobs()?;
        // Recheck after stopping, while run locks prevent an automatic resume.
        if self.goal_root(saved, slug)? != root {
            return Err("Goal folder changed".into());
        }
        let mut next = saved.clone();
        let changed = next.config.active_slug.as_deref() == Some(slug);
        next.config.slugs.retain(|g| g.slug != slug);
        if next.config.active_slug.as_deref() == Some(slug) {
            next.config.active_slug = None;
            next.config.goal.clear();
            next.config.goal_sources.clear();
            next.proposal = None;
            next.job = None;
            next.last_run = None;
            next.imports = None;
            next.mcp_result = None;
        }
        // Validate serializability before the irreversible filesystem operation.
        if serde_json::to_vec_pretty(&next)
            .map_err(|e| e.to_string())?
            .len()
            > 64 * 1024
        {
            return Err("Workspace settings exceed limit".into());
        }
        // Retained audit runs cannot be resumed from a stale Room request after deletion.
        for run in &runs {
            write(&run.join("goal-deleted.json"), &audit)?;
        }
        fs::remove_dir_all(&root)
            .map_err(|e| format!("Goal deletion failed; registry retained: {e}"))?;
        let mut current = self.saved.lock().map_err(|_| "Startup settings poisoned")?;
        self.store(&next)
            .map_err(|e| format!("Goal folder deleted, but registry update failed: {e}"))?;
        *current = next;
        crate::state::Store::open(self.broker.dir.join("state.sqlite"))?.record(
            "goal_deleted",
            &audit,
            "Permanently deleted the confirmed owned Goal folder",
        )?;
        Ok(serde_json::json!({"deleted":true,"stopping":false,"slug":slug,"changed":changed}))
    }
}

fn request_stop(run: &Path) -> Result<(), String> {
    match workflow::stop(run, "Human confirmed permanent Goal deletion") {
        Ok(()) => Ok(()),
        Err(e) if e.starts_with("unknown process ownership") => Ok(()),
        Err(e) => Err(e),
    }
}

fn verify_processes(root: &Path, remaining: &mut usize) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        *remaining = remaining
            .checked_sub(1)
            .ok_or("Goal process inventory exceeds limit")?;
        let entry = entry.map_err(|e| e.to_string())?;
        let meta = fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
        if !crate::folder::ordinary(&meta) {
            return Err("Goal run process inventory contains an alias".into());
        }
        if meta.is_dir() {
            verify_processes(&entry.path(), remaining)?;
        } else if entry.file_name() == "owner.json" {
            let owner: crate::process::Ownership = read(&entry.path())?;
            owner.ensure_quiescent()?;
        }
    }
    Ok(())
}
