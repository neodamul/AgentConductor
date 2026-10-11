//! DD-56: store only the agent list, keeping coordinator and role rules consistent.
use super::*;

impl Startup {
    pub(crate) fn profiles(&self) -> Result<Vec<AgentProfile>, String> {
        Ok(self
            .saved
            .lock()
            .map_err(|_| "startup settings poisoned")?
            .config
            .profiles
            .clone())
    }

    /// DD-57 amendment: reuse catalogs observed for the same CLI in another workspace.
    pub(crate) fn adopt_catalogs(&self, observed: &[AgentProfile]) -> Result<(), String> {
        let mut saved = self.saved.lock().map_err(|_| "startup settings poisoned")?;
        let mut next = saved.clone();
        let mut changed = false;
        for profile in next
            .config
            .profiles
            .iter_mut()
            .filter(|p| p.catalog.is_none())
        {
            if let Some(source) = observed.iter().find(|o| {
                o.catalog.is_some()
                    && o.parser == profile.parser
                    && o.executable == profile.executable
            }) {
                profile.catalog = source.catalog.clone();
                changed = true;
            }
        }
        if changed {
            self.store(&next)?;
            *saved = next;
        }
        Ok(())
    }

    pub(super) fn save_profiles(
        &self,
        saved: &mut Saved,
        mut profiles: Vec<AgentProfile>,
        renames: &std::collections::BTreeMap<String, String>,
    ) -> Result<serde_json::Value, String> {
        let mut next = saved.clone();
        // DD-74: references follow a renamed agent; references to removed agents are dropped.
        let rename = |id: &mut String| {
            if let Some(new) = renames.get(id.as_str()) {
                *id = new.clone();
            }
        };
        rename(&mut next.config.coordinator);
        for entries in next.config.roles.values_mut() {
            entries.iter_mut().for_each(|e| rename(&mut e.agent));
        }
        let ids: Vec<String> = profiles.iter().map(|p| p.id.clone()).collect();
        for profile in &mut profiles {
            if let Some(fallback) = &mut profile.fallback {
                rename(&mut fallback.agent);
                if !ids.contains(&fallback.agent) {
                    profile.fallback = None;
                }
            }
        }
        validate_profiles(&profiles)?;
        carry_over(&mut profiles, &saved.config.profiles);
        if !profiles.iter().any(|p| p.id == next.config.coordinator) {
            next.config.coordinator = profiles.first().map(|p| p.id.clone()).unwrap_or_default();
        }
        for entries in next.config.roles.values_mut() {
            entries.retain(|e| profiles.iter().any(|p| p.id == e.agent));
        }
        next.config.roles.retain(|_, entries| !entries.is_empty());
        next.config.profiles = profiles;
        if execution_view(&next.config.profiles)? != execution_view(&saved.config.profiles)? {
            next.proposal = None;
            next.job = None;
        }
        self.store(&next)?;
        *saved = next;
        Ok(serde_json::json!({"saved":true,"config":saved.config}))
    }
}

pub(crate) fn validate_name(name: &str) -> Result<(), String> {
    if name.chars().count() > 80 || name.chars().any(char::is_control) {
        return Err(
            "workspace name must be at most 80 characters without control characters".into(),
        );
    }
    Ok(())
}

/// Keeps private environment values and observed catalogs the page does not send back
/// (DD-56/DD-57). A catalog survives only for the same agent ID, parser and executable.
pub(super) fn carry_over(profiles: &mut [AgentProfile], stored: &[AgentProfile]) {
    for profile in profiles {
        let Some(old) = stored.iter().find(|p| p.id == profile.id) else {
            continue;
        };
        if profile.environment.is_empty() {
            profile.environment = old.environment.clone();
        }
        if profile.catalog.is_none()
            && old.parser == profile.parser
            && old.executable == profile.executable
        {
            profile.catalog = old.catalog.clone();
        }
    }
}

/// What a plan was reviewed against: every agent setting except its observed catalog.
pub(super) fn execution_view(agents: &[AgentProfile]) -> Result<Vec<u8>, String> {
    let mut agents = agents.to_vec();
    agents.iter_mut().for_each(|a| {
        a.catalog = None;
        a.quota_refresh_minutes = 5;
    });
    serde_json::to_vec(&agents).map_err(|e| e.to_string())
}

pub(crate) fn validate_profiles(profiles: &[AgentProfile]) -> Result<(), String> {
    // DD-72: an empty registry is valid; planning requires an agent separately.
    if profiles.len() > 16 {
        return Err("register at most 16 CLI profiles".into());
    }
    if profiles.is_empty() {
        return Ok(());
    }
    let probe = Plan {
        goal: "validate registration".into(),
        workspace_mode: WorkspaceMode::Shared,
        agents: profiles.to_vec(),
        tasks: (0..3)
            .map(|i| Task {
                id: format!("registration-{i}"),
                agent: profiles[0].id.clone(),
                prompt: "validation only".into(),
                paths: vec![format!("registration-{i}")],
                depends_on: vec![],
                test_command: vec!["true".into()],
                model: None,
                role: None,
            })
            .collect(),
    };
    probe.validate()?;
    for p in profiles {
        if p.executable.contains('\0')
            || p.args.iter().any(|a| {
                a.contains('\0')
                    || a.starts_with("--dangerously")
                    || matches!(a.as_str(), "--yolo" | "--auto")
            })
        {
            return Err("invalid CLI argv or permission bypass".into());
        }
        if !matches!(
            p.permission_mode.as_str(),
            "unverified" | "native_sandbox" | "native_callback" | "read_only" | "external_sandbox"
        ) {
            return Err("unknown permission capability".into());
        }
        if let Some(catalog) = &p.catalog {
            validate_catalog(catalog)?;
        }
    }
    Ok(())
}
pub(crate) fn validate_catalog(c: &ModelCatalog) -> Result<(), String> {
    if c.models.is_empty()
        || c.models.len() > 1024
        || c.queried_at <= 0
        || c.source.trim().is_empty()
        || c.cli_version.trim().is_empty()
        || c.models
            .iter()
            .any(|m| m.is_empty() || m.starts_with('-') || m.chars().any(char::is_whitespace))
    {
        return Err("catalog requires observed model IDs and query provenance".into());
    }
    Ok(())
}
