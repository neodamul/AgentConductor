//! DD-61: create the slug for the current goal and its work tree.
use super::*;

impl Startup {
    pub(super) fn create_slug(
        &self,
        saved: &mut Saved,
        requested: Option<String>,
    ) -> Result<serde_json::Value, String> {
        let goal = saved.config.goal.trim().to_string();
        if goal.is_empty() {
            return Err(
                crate::ui::messages::text("native.enter_a_goal_before_creating_its_slug").into(),
            );
        }
        let existing: Vec<String> = saved.config.slugs.iter().map(|s| s.slug.clone()).collect();
        let slug = match requested {
            Some(slug) => {
                crate::slugs::validate(&slug)?;
                if existing.contains(&slug) {
                    return Err(crate::ui::messages::render(
                        "native.slug_already_exists_in_this_workspace",
                        &[("slug", &slug.to_string())],
                    ));
                }
                slug
            }
            None => {
                // R-74 / DD-82: also avoid folders already on disk (for example after an import).
                let mut taken = existing.clone();
                loop {
                    let slug = crate::slugs::unique(&crate::slugs::suggest(&goal), &taken);
                    let root = crate::slugs::root(
                        &saved.config.repository,
                        &self.broker.dir,
                        &saved.config.name,
                        &slug,
                    );
                    if !root.exists() {
                        break slug;
                    }
                    taken.push(slug);
                }
            }
        };
        if let Ok(repo) = Repository::open(&saved.config.repository) {
            crate::slugs::exclude_from_git(&repo, &slug)?;
        }
        let root = crate::slugs::root(
            &saved.config.repository,
            &self.broker.dir,
            &saved.config.name,
            &slug,
        );
        crate::slugs::create_tree(&root)?;
        let mut next = saved.clone();
        next.config.slugs.push(crate::slugs::Slug {
            slug: slug.clone(),
            goal,
            last_run: None,
            created_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis()
                .try_into()
                .map_err(|_| crate::ui::messages::text("native.clock_out_of_range"))?,
        });
        next.config.active_slug = Some(slug.clone());
        self.store(&next)?;
        *saved = next;
        Ok(serde_json::json!({"slug":slug,"root":root}))
    }
}

/// DD-92: a date placeholder takes the coordinator's English slug when the plan is ready.
pub(super) fn adopt_planned_slug(
    config: &mut Config,
    broker: &Path,
    p: &mut Proposal,
) -> Result<(), String> {
    let Some(old) = p.slug.clone().filter(|v| crate::slugs::is_placeholder(v)) else {
        return Ok(());
    };
    let Ok(name) = fs::read_to_string(p.directory.join("slug.txt")) else {
        return Ok(());
    };
    if crate::slugs::validate(name.trim()).is_err() {
        return Ok(());
    }
    let root = |v: &str| crate::slugs::root(&config.repository, broker, &config.name, v);
    let mut taken: Vec<String> = config.slugs.iter().map(|v| v.slug.clone()).collect();
    let new = loop {
        let candidate = crate::slugs::unique(name.trim(), &taken);
        if !root(&candidate).exists() {
            break candidate;
        }
        taken.push(candidate);
    };
    // A failed rename (for example a locked folder) keeps the placeholder; settle must not fail.
    if fs::rename(root(&old), root(&new)).is_err() {
        return Ok(());
    }
    // The renamed goal folder sits in the project root and stays out of Git status.
    if let Ok(repo) = crate::git::Repository::open(&config.repository) {
        let _ = crate::slugs::exclude_from_git(&repo, &new);
    }
    for entry in config.slugs.iter_mut().filter(|v| v.slug == old) {
        entry.slug = new.clone();
    }
    if config.active_slug.as_deref() == Some(old.as_str()) {
        config.active_slug = Some(new.clone());
    }
    p.slug = Some(new);
    Ok(())
}

impl Startup {
    fn goal_run_idle(&self, saved: &Saved) -> Result<(), String> {
        if let Some(run) = self.last_run(saved) {
            let report = workflow::load(&run)?;
            if !matches!(report.state.as_str(), "completed" | "stopped" | "failed")
                || report.tasks.iter().any(|task| task.status == "running")
            {
                return Err(
                    messages::text("native.finish_or_stop_run_before_changing_goals").into(),
                );
            }
        }
        Ok(())
    }
    fn archive_goal_run(&self, saved: &mut Saved) -> Result<(), String> {
        let Some(run) = self.last_run(saved) else {
            return Ok(());
        };
        let report = workflow::load(&run)?;
        if let Some(slug) = saved.config.slugs.iter_mut().find(|slug| {
            Some(&slug.slug) == saved.config.active_slug.as_ref() && slug.goal == report.plan.goal
        }) {
            slug.last_run = Some(run);
        }
        Ok(())
    }
    pub(super) fn change_goal(
        &self,
        saved: &mut Saved,
        selected: Option<String>,
    ) -> Result<serde_json::Value, String> {
        self.goal_run_idle(saved)?;
        if let Some(slug) = &selected {
            crate::slugs::validate(slug)?;
            if saved.config.active_slug.as_ref() == Some(slug) {
                return Ok(serde_json::json!({"changed":false}));
            }
            if !saved.config.slugs.iter().any(|item| &item.slug == slug) {
                return Err(messages::text("native.goal_is_not_registered").into());
            }
        } else if saved.config.active_slug.is_none()
            && !saved.config.goal.trim().is_empty()
            && self.last_run(saved).is_some()
        {
            self.create_slug(saved, None)?;
        }
        let mut next = saved.clone();
        self.archive_goal_run(&mut next)?;
        next.config.active_slug = selected.clone();
        next.last_run = None;
        next.config.goal.clear();
        if let Some(slug) = selected {
            let record = next
                .config
                .slugs
                .iter()
                .find(|item| item.slug == slug)
                .unwrap();
            next.config.goal = record.goal.clone();
            if let Some(run) = &record.last_run {
                let run = run.canonicalize().map_err(|e| e.to_string())?;
                if !run.starts_with(self.broker.dir.canonicalize().map_err(|e| e.to_string())?)
                    || workflow::load(&run)?.plan.goal != record.goal
                {
                    return Err(messages::text("native.goal_run_binding_changed").into());
                }
                next.last_run = Some(run);
            }
        }
        next.config.goal_sources.clear();
        next.proposal = None;
        next.job = None;
        next.imports = None;
        next.mcp_result = None;
        self.store(&next)?;
        *saved = next;
        Ok(serde_json::json!({"changed":true}))
    }
}
