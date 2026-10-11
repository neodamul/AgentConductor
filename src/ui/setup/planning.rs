//! DD-60: coordinator planning from a goal or from an existing plan document.
use super::*;

const MAX_DOCUMENT: u64 = 64 * 1024;

impl Startup {
    pub(super) fn start_planning(
        &self,
        saved: &mut Saved,
        reference: Option<&Path>,
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
        let config = &saved.config;
        if config.goal.trim().is_empty() {
            return Err("enter a nonempty goal".into());
        }
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
        if let Some(path) = reference {
            args.push("--from".into());
            args.push(path.to_string_lossy().into());
        }
        saved.proposal = Some(Proposal {
            directory,
            repository_base: repo.head()?,
            run,
            phase: "planning".into(),
        });
        self.store(saved)?;
        self.launch(saved, "plan", args)
    }
}
