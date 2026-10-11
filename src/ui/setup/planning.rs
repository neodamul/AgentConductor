//! DD-60: coordinator planning from a goal or from an existing plan document.
use super::*;

const MAX_DOCUMENT: u64 = 64 * 1024;

impl Startup {
    pub(super) fn start_planning(
        &self,
        saved: &mut Saved,
        reference: Option<&Path>,
        slug: Option<String>,
    ) -> Result<serde_json::Value, String> {
        if let Some(path) = reference {
            let size = fs::metadata(path)
                .map_err(|e| format!("plan document unavailable: {e}"))?
                .len();
            if size > MAX_DOCUMENT {
                return Err("plan document exceeds 64 KiB".into());
            }
            if saved.config.goal.trim().is_empty() {
                // An empty goal takes the document's first heading.
                let text =
                    fs::read_to_string(path).map_err(|_| "plan document is not UTF-8 text")?;
                let heading = text
                    .lines()
                    .find_map(|l| l.trim().strip_prefix('#'))
                    .map(|h| h.trim_start_matches('#').trim().to_string())
                    .filter(|h| !h.is_empty())
                    .ok_or("enter a goal; the plan document has no heading")?;
                let mut next = saved.clone();
                next.config.goal = heading;
                self.store(&next)?;
                *saved = next;
            }
        }
        if saved.config.goal.trim().is_empty() {
            return Err("enter a nonempty goal".into());
        }
        // DD-61: a new goal gets its own slug; the active slug is reused for the same goal.
        let active = saved
            .config
            .active_slug
            .as_ref()
            .and_then(|a| saved.config.slugs.iter().find(|s| &s.slug == a))
            .filter(|s| {
                s.goal == saved.config.goal.trim() && slug.as_ref().is_none_or(|w| w == &s.slug)
            })
            .map(|s| s.slug.clone());
        let slug = match active {
            Some(slug) => slug,
            None => self.create_slug(saved, slug)?["slug"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        };
        let config = &saved.config;
        validate_profiles(&config.profiles)?;
        let coordinator = config
            .profiles
            .iter()
            .find(|p| p.id == config.coordinator)
            .ok_or("coordinator not registered")?;
        crate::coordinator::planning_args(coordinator, "{task_file}")?;
        let repo = Repository::open(&config.repository)?;
        repo.clean()?;
        if self.broker.dir.starts_with(&repo.root) {
            return Err("startup artifacts must be outside repository".into());
        }
        write(&self.broker.dir.join("profiles.json"), &config.profiles)?;
        write(&self.broker.dir.join("roles.json"), &config.roles)?;
        let rules = self.broker.dir.join("plan-rules.md");
        fs::write(
            &rules,
            crate::plan_rules::current(config.plan_rules.as_deref()),
        )
        .map_err(|e| e.to_string())?;
        let slug_root = crate::slugs::root(&config.repository, &self.broker.dir, &slug);
        let directory = self
            .broker
            .dir
            .join(format!("plan-{}", crate::native_guard::session_id()?));
        let run = self
            .broker
            .dir
            .join(format!("run-{}", crate::native_guard::session_id()?));
        let mut args = vec![
            "plan".into(),
            self.broker
                .dir
                .join("profiles.json")
                .to_string_lossy()
                .into(),
            config.coordinator.clone(),
            repo.root.to_string_lossy().into(),
            directory.to_string_lossy().into(),
            config.goal.clone(),
            self.broker.dir.join("roles.json").to_string_lossy().into(),
        ];
        args.extend([
            "--rules".into(),
            rules.to_string_lossy().into(),
            "--phases".into(),
            slug_root.to_string_lossy().into(),
        ]);
        if let Some(path) = reference {
            args.push("--from".into());
            args.push(path.to_string_lossy().into());
        }
        saved.proposal = Some(Proposal {
            directory,
            repository_base: repo.head()?,
            run,
            phase: "planning".into(),
            slug: Some(slug),
            slug_phase: None,
        });
        self.store(saved)?;
        self.launch(saved, "plan", args)
    }
}

impl Startup {
    /// DD-62: the next phase comes from the slug tree; no coordinator runs.
    pub(super) fn next_phase(&self, saved: &mut Saved) -> Result<serde_json::Value, String> {
        let slug = saved.config.active_slug.clone().ok_or("no active slug")?;
        let root = crate::slugs::root(&saved.config.repository, &self.broker.dir, &slug);
        let phase = crate::slug_progress::progress(&root)["current_phase"]
            .as_str()
            .map(str::to_owned)
            .ok_or("every phase of this slug is completed")?;
        let base = Plan {
            goal: saved.config.goal.clone(),
            agents: saved.config.profiles.clone(),
            tasks: vec![],
            workspace_mode: saved.config.workspace_mode,
        };
        let plan = crate::phases::run_plan(&root, &phase, &base)?;
        let repo = Repository::open(&saved.config.repository)?;
        repo.clean()?;
        let directory = self
            .broker
            .dir
            .join(format!("plan-{}", crate::native_guard::session_id()?));
        fs::create_dir(&directory).map_err(|e| e.to_string())?;
        write(&directory.join("plan.json"), &plan)?;
        write(&directory.join("review.json"), &plan)?;
        let mut next = saved.clone();
        next.proposal = Some(Proposal {
            directory,
            repository_base: repo.head()?,
            run: self
                .broker
                .dir
                .join(format!("run-{}", crate::native_guard::session_id()?)),
            phase: "ready".into(),
            slug: Some(slug),
            slug_phase: Some(phase.clone()),
        });
        next.job = None;
        self.store(&next)?;
        *saved = next;
        Ok(serde_json::json!({"phase":phase}))
    }

    /// DD-65: workspace override (None clears it) or the global rules file.
    pub(super) fn save_plan_rules(
        &self,
        saved: &mut Saved,
        scope: &str,
        text: Option<String>,
    ) -> Result<serde_json::Value, String> {
        if text.as_ref().is_some_and(|t| t.len() > 64 * 1024) {
            return Err("plan rules exceed 64 KiB".into());
        }
        match scope {
            "global" => {
                crate::plan_rules::save_global(text.as_deref().ok_or("enter the global rules")?)?
            }
            "workspace" => {
                let mut next = saved.clone();
                next.config.plan_rules = text;
                self.store(&next)?;
                *saved = next;
            }
            _ => return Err("scope must be global or workspace".into()),
        }
        Ok(serde_json::json!({"saved":true}))
    }

    /// Slugs with progress derived from their trees (DD-63).
    pub(super) fn slug_snapshot(&self, saved: &Saved) -> serde_json::Value {
        saved
            .config
            .slugs
            .iter()
            .map(|s| {
                let root = crate::slugs::root(&saved.config.repository, &self.broker.dir, &s.slug);
                serde_json::json!({"slug":s.slug,"goal":s.goal,"active":saved.config.active_slug.as_deref() == Some(s.slug.as_str()),"progress":crate::slug_progress::progress(&root)})
            })
            .collect()
    }
}
