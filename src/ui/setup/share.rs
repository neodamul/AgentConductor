//! DD-59: workspace settings as a Git-friendly file, without secrets or machine state.
use super::*;

const FILE: &str = "agentconductor.workspace.json";
const FORMAT: &str = "agentconductor.workspace/v1";
/// Capabilities asserted by an operator about their own machine (DD-17).
const LOCAL_CLAIMS: [&str; 2] = ["external_sandbox", "read_only"];

fn folder(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err("choose an absolute folder".into());
    }
    path.canonicalize()
        .map_err(|e| format!("folder unavailable: {e}"))
}

fn read_shared(dir: &Path) -> Result<serde_json::Value, String> {
    let path = folder(dir)?.join(FILE);
    let text = fs::read_to_string(&path).map_err(|_| format!("no {FILE} in that folder"))?;
    if text.len() > 256 * 1024 {
        return Err("workspace settings file exceeds 256 KiB".into());
    }
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("invalid {FILE}: {e}"))?;
    if value["format"] != FORMAT {
        return Err(format!("unsupported settings format; expected {FORMAT}"));
    }
    Ok(value)
}

impl Startup {
    pub(super) fn export_settings(
        &self,
        saved: &Saved,
        dir: &Path,
    ) -> Result<serde_json::Value, String> {
        let dir = folder(dir)?;
        let config = &saved.config;
        let agents: Vec<serde_json::Value> = config
            .profiles
            .iter()
            .map(|p| {
                let mut value = serde_json::to_value(p).map_err(|e| e.to_string())?;
                let object = value.as_object_mut().ok_or("invalid profile")?;
                object.remove("environment");
                object.remove("catalog");
                Ok(value)
            })
            .collect::<Result<_, String>>()?;
        let mut file = serde_json::json!({
            "format": FORMAT,
            "name": config.name,
            "coordinator": config.coordinator,
            "workspace_mode": config.workspace_mode,
            "agents": agents,
            "roles": config.roles,
        });
        if !config.repository.as_os_str().is_empty() && config.repository == dir {
            file["repository"] = ".".into();
        }
        let mut text = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
        text.push('\n');
        let path = dir.join(FILE);
        fs::write(&path, text).map_err(|e| e.to_string())?;
        Ok(serde_json::json!({"exported":path}))
    }

    pub(super) fn preview_settings(
        &self,
        saved: &Saved,
        dir: &Path,
    ) -> Result<serde_json::Value, String> {
        let value = read_shared(dir)?;
        let agents: Vec<AgentProfile> = serde_json::from_value(value["agents"].clone())
            .map_err(|e| format!("invalid agents: {e}"))?;
        let mut warnings = vec![];
        for agent in &agents {
            if LOCAL_CLAIMS.contains(&agent.permission_mode.as_str()) {
                warnings.push(format!("{}: {} must be verified on this machine; it is imported as Not verified unless you confirm", agent.id, agent.permission_mode));
            }
            if Path::new(&agent.executable).is_absolute() {
                warnings.push(format!(
                    "{}: absolute executable {} may not exist on this machine",
                    agent.id, agent.executable
                ));
            }
        }
        for local in &saved.config.profiles {
            if !agents.iter().any(|a| a.id == local.id) {
                warnings.push(format!(
                    "{}: local agent is not in the shared settings and will be removed",
                    local.id
                ));
            }
        }
        Ok(serde_json::json!({"settings":value,"warnings":warnings}))
    }

    pub(super) fn import_settings(
        &self,
        saved: &mut Saved,
        dir: &Path,
        keep_verified: bool,
    ) -> Result<serde_json::Value, String> {
        let value = read_shared(dir)?;
        let mut profiles: Vec<AgentProfile> = serde_json::from_value(value["agents"].clone())
            .map_err(|e| format!("invalid agents: {e}"))?;
        validate_profiles(&profiles)?;
        for profile in &mut profiles {
            profile.environment.clear();
            profile.catalog = None;
            if !keep_verified && LOCAL_CLAIMS.contains(&profile.permission_mode.as_str()) {
                profile.permission_mode = "unverified".into();
            }
        }
        profiles::carry_over(&mut profiles, &saved.config.profiles);
        let roles: crate::roles::RoleRules = serde_json::from_value(value["roles"].clone())
            .map_err(|e| format!("invalid roles: {e}"))?;
        crate::roles::validate(&roles, &profiles)?;
        let coordinator = value["coordinator"].as_str().ok_or("missing coordinator")?;
        if !profiles.iter().any(|p| p.id == coordinator) {
            return Err("shared coordinator is not among the shared agents".into());
        }
        let name = value["name"].as_str().unwrap_or_default();
        validate_name(name)?;
        let mut next = saved.clone();
        next.config.workspace_mode = serde_json::from_value(value["workspace_mode"].clone())
            .map_err(|e| format!("invalid workspace mode: {e}"))?;
        if value["repository"] == "." {
            next.config.repository = Repository::open(&folder(dir)?)?.root;
        }
        next.config.name = name.into();
        next.config.coordinator = coordinator.into();
        next.config.roles = roles;
        next.config.profiles = profiles;
        // DD-58: keep a plan only if what it was reviewed against is unchanged.
        let keep = next.config.repository == saved.config.repository
            && serde_json::to_value(next.config.workspace_mode).ok()
                == serde_json::to_value(saved.config.workspace_mode).ok()
            && profiles::execution_view(&next.config.profiles)?
                == profiles::execution_view(&saved.config.profiles)?;
        if !keep {
            next.proposal = None;
            next.job = None;
        }
        self.store(&next)?;
        *saved = next;
        Ok(serde_json::json!({"imported":true}))
    }
}
