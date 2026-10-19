//! DD-60: coordinator planning from a goal or from an existing plan document.
use super::*;

const MAX_DOCUMENT: u64 = 64 * 1024;

impl Startup {
    pub(super) fn stop_planning(&self, saved: &mut Saved) -> Result<serde_json::Value, String> {
        if !saved.metadata_queue.is_empty() {
            saved.metadata_queue.clear();
            self.store(saved)?;
        }
        if saved.job.as_ref().is_none_or(|j| j.operation != "plan") {
            return Ok(serde_json::json!({"stopping":false}));
        }
        let proposal = saved.proposal.as_ref().ok_or(crate::ui::messages::text(
            "native.planning_proposal_unavailable",
        ))?;
        if proposal.directory.parent() != Some(self.broker.dir.as_path()) {
            return Err(
                crate::ui::messages::text("native.planning_stop_path_outside_workspace").into(),
            );
        }
        let marker = proposal.directory.with_extension("stop.request");
        if !marker.exists() {
            write(
                &marker,
                &serde_json::json!({"decision":"human_stop_planning"}),
            )?;
        }
        Ok(serde_json::json!({"stopping":true}))
    }

    pub(super) fn start_planning(
        &self,
        saved: &mut Saved,
        reference: Option<&Path>,
        slug: Option<String>,
    ) -> Result<serde_json::Value, String> {
        if let Some(path) = reference {
            let size = fs::metadata(path)
                .map_err(|e| {
                    crate::ui::messages::render(
                        "native.plan_document_unavailable",
                        &[("e", &e.to_string())],
                    )
                })?
                .len();
            if size > MAX_DOCUMENT {
                return Err(
                    crate::ui::messages::text("native.plan_document_exceeds_64_kib").into(),
                );
            }
            if saved.config.goal.trim().is_empty() {
                // An empty goal takes the document's first heading.
                let text = fs::read_to_string(path).map_err(|_| {
                    crate::ui::messages::text("native.plan_document_is_not_utf_8_text")
                })?;
                let heading = text
                    .lines()
                    .find_map(|l| l.trim().strip_prefix('#'))
                    .map(|h| h.trim_start_matches('#').trim().to_string())
                    .filter(|h| !h.is_empty())
                    .ok_or(crate::ui::messages::text(
                        "native.enter_a_goal_the_plan_document_has_no",
                    ))?;
                let mut next = saved.clone();
                next.config.goal = heading;
                self.store(&next)?;
                *saved = next;
            }
        }
        let imported_context = self.source_context(saved)?;
        if saved.config.goal.trim().is_empty() {
            return Err(crate::ui::messages::text("native.enter_a_nonempty_goal").into());
        }
        // Validate the baseline before creating any goal folders or launching a CLI.
        let repo = self.execution_repository(saved)?;
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
        // DD-62: a goal that already has a plan is never replanned; its current phase is
        // loaded for review with the current profiles instead.
        if reference.is_none()
            && crate::slugs::root(
                &saved.config.repository,
                &self.broker.dir,
                &saved.config.name,
                &slug,
            )
            .join("plan/plan.json")
            .exists()
        {
            return self.next_phase(saved);
        }
        let config = &saved.config;
        validate_profiles(&config.profiles)?;
        let coordinator = config
            .profiles
            .iter()
            .find(|p| p.id == config.coordinator)
            .ok_or(crate::ui::messages::text(
                "native.coordinator_not_registered",
            ))?;
        crate::coordinator::planning_args(coordinator, "{task_file}")?;
        repo.clean()?;
        if self.broker.dir.starts_with(&repo.root) {
            return Err(crate::ui::messages::text(
                "native.startup_artifacts_must_be_outside_repository",
            )
            .into());
        }
        write(&self.broker.dir.join("profiles.json"), &config.profiles)?;
        write(&self.broker.dir.join("roles.json"), &config.roles)?;
        let rules = self.broker.dir.join("plan-rules.md");
        fs::write(
            &rules,
            crate::plan_rules::for_repository(config.plan_rules.as_deref(), &repo.root)?,
        )
        .map_err(|e| e.to_string())?;
        let slug_root =
            crate::slugs::root(&config.repository, &self.broker.dir, &config.name, &slug);
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
        if let Some(context) = imported_context {
            let path = self.broker.dir.join(format!(
                "source-context-{}.md",
                crate::native_guard::session_id()?
            ));
            fs::write(&path, context).map_err(|e| e.to_string())?;
            args.extend(["--context".into(), path.to_string_lossy().into()]);
        }
        saved.proposal = Some(Proposal {
            workspace: Some(repo.root.clone()),
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
        let slug = saved
            .config
            .active_slug
            .clone()
            .ok_or(crate::ui::messages::text("native.no_active_slug"))?;
        let root = crate::slugs::root(
            &saved.config.repository,
            &self.broker.dir,
            &saved.config.name,
            &slug,
        );
        let phase = crate::slug_progress::progress(&root)["current_phase"]
            .as_str()
            .map(str::to_owned)
            .ok_or(crate::ui::messages::text(
                "native.every_phase_of_this_slug_is_completed",
            ))?;
        let base = Plan {
            goal: saved.config.goal.clone(),
            agents: saved.config.profiles.clone(),
            tasks: vec![],
            workspace_mode: saved.config.workspace_mode,
        };
        let plan = crate::phases::run_plan(&root, &phase, &base)?;
        let repo = self.execution_repository(saved)?;
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
            workspace: Some(repo.root.clone()),
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
            return Err(crate::ui::messages::text("native.plan_rules_exceed_64_kib").into());
        }
        match scope {
            "global" => crate::plan_rules::save_global(
                text.as_deref()
                    .ok_or(crate::ui::messages::text("native.enter_the_global_rules"))?,
            )?,
            "workspace" => {
                let mut next = saved.clone();
                next.config.plan_rules = text;
                self.store(&next)?;
                *saved = next;
            }
            _ => {
                return Err(
                    crate::ui::messages::text("native.scope_must_be_global_or_workspace").into(),
                );
            }
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
                let root = crate::slugs::root(&saved.config.repository, &self.broker.dir, &saved.config.name, &s.slug);
                let active = saved.config.active_slug.as_deref() == Some(s.slug.as_str());
                // DD-125: the active goal carries its whole phase plan for review.
                let plan = if active { crate::slug_progress::briefing(&root) } else { serde_json::Value::Null };
                serde_json::json!({"slug":s.slug,"goal":s.goal,"active":active,"progress":crate::slug_progress::progress(&root),"plan":plan})
            })
            .collect()
    }
}
