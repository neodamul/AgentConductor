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
                        &[("slug", &format!("{}", slug))],
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
