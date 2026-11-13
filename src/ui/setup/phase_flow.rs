//! #943: use existing background jobs; no additional polling loop or resident worker.
use super::*;

impl Startup {
    pub(super) fn phase_requires_replan(&self, root: &Path, phase: &str) -> Result<bool, String> {
        let index = crate::phase_replan::metadata(&root.join("plan/plan.json"))?;
        Ok(index["phases"]
            .as_array()
            .and_then(|p| p.iter().position(|p| p["id"] == phase))
            .is_some_and(|at| at > 0))
    }
    pub(super) fn launch_phase_replan(
        &self,
        saved: &mut Saved,
        root: &Path,
        phase: &str,
        original: Plan,
    ) -> Result<serde_json::Value, String> {
        if let Some(p) = &saved.proposal
            && p.slug_phase.as_deref() == Some(phase)
            && p.phase == "planning"
            && self.job(saved)?["state"] == "running"
        {
            return Ok(serde_json::json!({"accepted":true,"phase":phase}));
        }
        if let Some(p) = &saved.proposal
            && p.slug_phase.as_deref() == Some(phase)
            && p.phase == "ready"
            && p.directory.join("replan.json").exists()
        {
            return Ok(serde_json::json!({"phase":phase}));
        }
        let directory = self
            .broker
            .dir
            .join(format!("plan-{}", crate::native_guard::session_id()?));
        let request = crate::phase_replan::Request {
            root: root.into(),
            phase: phase.into(),
            original,
            coordinator: saved.config.coordinator.clone(),
            repository: saved.config.repository.clone(),
            directory: directory.clone(),
            roles: saved.config.roles.clone(),
            rules: crate::plan_rules::for_repository(
                saved.config.plan_rules.as_deref(),
                &saved.config.repository,
            )?,
        };
        let input = directory.with_extension("replan-input.json");
        write(&input, &request)?;
        saved.proposal = Some(Proposal {
            workspace: None,
            directory,
            repository_base: String::new(),
            run: self
                .broker
                .dir
                .join(format!("run-{}", crate::native_guard::session_id()?)),
            phase: "planning".into(),
            slug: saved.config.active_slug.clone(),
            slug_phase: Some(phase.into()),
        });
        self.store(saved)?;
        self.launch(
            saved,
            "plan",
            vec!["phase-replan".into(), input.to_string_lossy().into()],
        )
    }
    pub(super) fn continue_unchanged_phase(&self, saved: &mut Saved) -> Result<(), String> {
        if self.broker.dir.join("phase-chain.stop").exists() {
            return Ok(());
        }
        let Some(p) = &saved.proposal else {
            return Ok(());
        };
        let metadata = p.directory.join("replan.json");
        if !metadata.exists() {
            return Ok(());
        }
        let replan = crate::phase_replan::metadata(&metadata)?;
        if replan["unchanged"] != true {
            return Ok(());
        }
        let plan: Plan = read(&p.directory.join("review.json"))?;
        let baseline: Plan = read(&p.directory.join("baseline.json"))?;
        if crate::phase_replan::compare(&baseline, &plan)?["unchanged"] != true {
            return Ok(());
        }
        let digest = workflow::digest(&serde_json::to_vec(&plan).map_err(|e| e.to_string())?);
        write(
            &p.directory.join("auto-consent.json"),
            &serde_json::json!({"digest":digest,"source":"unchanged_phase"}),
        )?;
        self.start_plan_as(saved, digest, self.revision(saved)?, true)?;
        Ok(())
    }
    pub(super) fn phase_tick(
        &self,
        saved: &mut Saved,
        job: &serde_json::Value,
    ) -> Result<(), String> {
        if job["state"] == "running"
            || self.broker.poll()?["state"] == "running"
            || self.broker.dir.join("phase-chain.stop").exists()
        {
            return Ok(());
        }
        let Some(proposal) = &saved.proposal else {
            return Ok(());
        };
        if proposal.phase != "starting" || proposal.slug_phase.is_none() {
            return Ok(());
        }
        let Some(run) = self.last_run(saved) else {
            return Ok(());
        };
        let Ok(report) = workflow::load(&run) else {
            return Ok(());
        };
        if report.state == "waiting_approval" && saved.config.auto_merge {
            let attempted = run.with_extension("auto-merge-attempt.json");
            let revision = workflow::revision(&report)?;
            if read::<serde_json::Value>(&attempted)
                .ok()
                .is_some_and(|last| last["revision"] == revision)
            {
                return Ok(());
            }
            let Ok(lease) = crate::leases::Lease::acquire(&run.join("coordinator.lock")) else {
                return Ok(());
            };
            drop(lease);
            write(
                &attempted,
                &serde_json::json!({"candidate":report.candidate,"revision":revision,"attempted":true}),
            )?;
            self.launch(
                saved,
                "automatic_merge",
                vec!["phase-auto-merge".into(), run.to_string_lossy().into()],
            )?;
            return Ok(());
        }
        if report.state != "completed" || report.error.is_some() {
            return Ok(());
        }
        let Some(slug) = &proposal.slug else {
            return Ok(());
        };
        let root = crate::slugs::root(
            &saved.config.repository,
            &self.broker.dir,
            &saved.config.name,
            slug,
        );
        if !crate::slug_progress::progress(&root)["current_phase"].is_string() {
            return Ok(());
        }
        // The previous job must release its workflow/process leases before a new job starts.
        if self.ensure_no_live_jobs().is_err() {
            return Ok(());
        }
        self.next_phase(saved)?;
        Ok(())
    }
    pub(crate) fn automatic_phase_job(&self) -> Option<serde_json::Value> {
        let saved = self.saved.try_lock().ok()?;
        if saved.job.as_ref()?.operation != "automatic_merge" {
            return None;
        }
        let job = self.job(&saved).ok()?;
        (job["state"] == "running" || job["state"] == "failed").then_some(job)
    }
    pub(crate) fn phase_transition(&self, run: &Path) -> Option<serde_json::Value> {
        let saved = self.saved.try_lock().ok()?;
        let proposal = saved.proposal.as_ref()?;
        if proposal.run == run
            || !proposal
                .directory
                .with_extension("replan-input.json")
                .exists()
        {
            return None;
        }
        Some(serde_json::json!({"phase":proposal.phase,"run":proposal.run}))
    }
    pub(crate) fn pause_phase_chain(&self, reason: &str) -> Result<(), String> {
        let mut saved = self.saved.lock().map_err(|_| "startup settings poisoned")?;
        write(
            &self.broker.dir.join("phase-chain.stop"),
            &serde_json::json!({"paused":true}),
        )?;
        crate::state::Store::open(self.broker.dir.join("state.sqlite"))?.record(
            "phase_chain_stop",
            &serde_json::json!({"reason":reason,"run":saved.last_run}),
            "Operator paused automatic Phase progression",
        )?;
        self.stop_planning(&mut saved)?;
        Ok(())
    }
}
