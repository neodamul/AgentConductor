//! DD-59: workspace settings as a Git-friendly file, without secrets or machine state.
use super::*;

const FILE: &str = "agentconductor.workspace.json";
const FORMAT: &str = "agentconductor.workspace/v1";
/// Capabilities asserted by an operator about their own machine (DD-17).
const LOCAL_CLAIMS: [&str; 2] = ["external_sandbox", "read_only"];

fn folder(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err(crate::ui::messages::text("native.choose_an_absolute_folder").into());
    }
    path.canonicalize().map_err(|e| {
        crate::ui::messages::render("native.folder_unavailable", &[("e", &format!("{}", e))])
    })
}

fn read_shared(dir: &Path) -> Result<serde_json::Value, String> {
    let path = folder(dir)?.join(FILE);
    let text = fs::read_to_string(&path).map_err(|_| {
        crate::ui::messages::render(
            "native.no_in_that_folder",
            &[("FILE", &format!("{}", FILE))],
        )
    })?;
    if text.len() > 256 * 1024 {
        return Err(
            crate::ui::messages::text("native.workspace_settings_file_exceeds_256_kib").into(),
        );
    }
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        crate::ui::messages::render(
            "native.invalid",
            &[("FILE", &format!("{}", FILE)), ("e", &format!("{}", e))],
        )
    })?;
    if value["format"] != FORMAT {
        return Err(crate::ui::messages::render(
            "native.unsupported_settings_format_expected",
            &[("FORMAT", &format!("{}", FORMAT))],
        ));
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
                let object = value
                    .as_object_mut()
                    .ok_or(crate::ui::messages::text("native.invalid_profile"))?;
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
            // DD-79: messenger settings travel; the webhook URL never does.
            "messenger": config.messenger,
            "mcp":config.mcp,
            "goal_sources":config.goal_sources,
        });
        if !config.repository.as_os_str().is_empty() && config.repository == dir {
            file["repository"] = ".".into();
        }
        // DD-64: one store-only archive per slug, listed with its derived progress.
        let mut listed = vec![];
        for entry in &config.slugs {
            let root = crate::slugs::root(
                &config.repository,
                &self.broker.dir,
                &config.name,
                &entry.slug,
            );
            if !root.is_dir() {
                return Err(crate::ui::messages::render(
                    "native.slug_has_no_work_tree_to_export",
                    &[("value1", &format!("{}", entry.slug))],
                ));
            }
            let archive = format!("{}.zip", entry.slug);
            crate::archive::pack(&root, &dir.join(&archive))?;
            listed.push(serde_json::json!({
                "slug": entry.slug, "goal": entry.goal, "created_ms": entry.created_ms,
                "archive": archive, "progress": crate::slug_progress::progress(&root),
            }));
        }
        file["slugs"] = listed.into();
        file["active_slug"] = serde_json::json!(config.active_slug);
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
        let agents: Vec<AgentProfile> =
            serde_json::from_value(value["agents"].clone()).map_err(|e| {
                crate::ui::messages::render("native.invalid_agents", &[("e", &format!("{}", e))])
            })?;
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
            .map_err(|e| {
                crate::ui::messages::render("native.invalid_agents", &[("e", &format!("{}", e))])
            })?;
        validate_profiles(&profiles)?;
        for profile in &mut profiles {
            profile.environment.clear();
            profile.catalog = None;
            if !keep_verified && LOCAL_CLAIMS.contains(&profile.permission_mode.as_str()) {
                profile.permission_mode = "unverified".into();
            }
        }
        profiles::carry_over(&mut profiles, &saved.config.profiles);
        profiles.iter_mut().for_each(profiles::derive_permission);
        let roles: crate::roles::RoleRules = serde_json::from_value(value["roles"].clone())
            .map_err(|e| {
                crate::ui::messages::render("native.invalid_roles", &[("e", &format!("{}", e))])
            })?;
        crate::roles::validate(&roles, &profiles)?;
        let coordinator = value["coordinator"]
            .as_str()
            .ok_or(crate::ui::messages::text("native.missing_coordinator"))?;
        if !(profiles.is_empty() && coordinator.is_empty())
            && !profiles.iter().any(|p| p.id == coordinator)
        {
            return Err(crate::ui::messages::text(
                "native.shared_coordinator_is_not_among_the_shared_agents",
            )
            .into());
        }
        let name = value["name"].as_str().unwrap_or_default();
        validate_name(name)?;
        let mut next = saved.clone();
        next.config.workspace_mode = serde_json::from_value(value["workspace_mode"].clone())
            .map_err(|e| {
                crate::ui::messages::render(
                    "native.invalid_workspace_mode",
                    &[("e", &format!("{}", e))],
                )
            })?;
        if value["repository"] == "." {
            next.config.repository = Repository::open(&folder(dir)?)?.root;
        }
        let settings=crate::mcp::Settings::parse(&serde_json::json!({"mcp":value.get("mcp").cloned().unwrap_or(serde_json::json!({})),"goal_sources":value.get("goal_sources").cloned().unwrap_or(serde_json::json!({}))}).to_string())?;
        if settings.digest()? != saved.config.mcp_settings().digest()? {
            next.imports = None;
            next.mcp_result = None;
            next.proposal = None;
            next.job = None;
        }
        next.config.mcp = settings.mcp;
        next.config.goal_sources = settings.goal_sources;
        next.config.name = name.into();
        next.config.coordinator = coordinator.into();
        next.config.roles = roles;
        if !value["messenger"].is_null() {
            next.config.messenger =
                serde_json::from_value(value["messenger"].clone()).map_err(|e| {
                    crate::ui::messages::render(
                        "native.invalid_messenger_settings",
                        &[("e", &format!("{}", e))],
                    )
                })?;
        }
        next.config.profiles = profiles;
        let imported = self.stage_slugs(&value, dir, &next.config.repository, &next.config.name)?;
        let names: Vec<String> = imported.iter().map(|(_, _, s)| s.slug.clone()).collect();
        for (staging, destination, _) in &imported {
            if destination.exists() {
                let backup = destination.with_extension("bak");
                let _ = fs::remove_dir_all(&backup);
                fs::rename(destination, &backup).map_err(|e| e.to_string())?;
            }
            fs::rename(staging, destination).map_err(|e| e.to_string())?;
        }
        if !imported.is_empty() && !next.config.repository.as_os_str().is_empty() {
            let repo = Repository::open(&next.config.repository)?;
            for slug in &names {
                crate::slugs::exclude_from_git(&repo, slug)?;
            }
        }
        let local: Vec<_> = saved
            .config
            .slugs
            .iter()
            .filter(|s| !names.contains(&s.slug))
            .cloned()
            .collect();
        next.config.slugs = imported
            .into_iter()
            .map(|(_, _, s)| s)
            .chain(local)
            .collect();
        if let Some(active) = value["active_slug"].as_str() {
            next.config.active_slug = Some(active.into());
        }
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

type StagedSlug = (PathBuf, PathBuf, crate::slugs::Slug);

impl Startup {
    /// DD-64: extract and verify every listed archive before anything is replaced.
    fn stage_slugs(
        &self,
        value: &serde_json::Value,
        dir: &Path,
        repository: &Path,
        name: &str,
    ) -> Result<Vec<StagedSlug>, String> {
        let mut staged: Vec<StagedSlug> = vec![];
        let result = (|| {
            for item in value["slugs"].as_array().into_iter().flatten() {
                let slug = item["slug"].as_str().ok_or(crate::ui::messages::text(
                    "native.slug_entry_without_a_name",
                ))?;
                crate::slugs::validate(slug)?;
                let archive = item["archive"].as_str().ok_or(crate::ui::messages::text(
                    "native.slug_entry_without_an_archive",
                ))?;
                if archive != format!("{slug}.zip") {
                    return Err(crate::ui::messages::render(
                        "native.archive_does_not_match_slug",
                        &[
                            ("archive", &format!("{}", archive)),
                            ("slug", &format!("{}", slug)),
                        ],
                    ));
                }
                let zip = folder(dir)?.join(archive);
                if !zip.is_file() {
                    return Err(format!("{archive} must be next to {FILE}"));
                }
                let destination = crate::slugs::root(repository, &self.broker.dir, name, slug);
                fs::create_dir_all(
                    destination
                        .parent()
                        .ok_or(crate::ui::messages::text("native.invalid_slug_root"))?,
                )
                .map_err(|e| e.to_string())?;
                let staging = destination.with_extension(format!("import-{}", std::process::id()));
                let _ = fs::remove_dir_all(&staging);
                crate::archive::unpack(&zip, &staging)?;
                staged.push((
                    staging.clone(),
                    destination,
                    crate::slugs::Slug {
                        slug: slug.into(),
                        goal: item["goal"].as_str().unwrap_or_default().into(),
                        created_ms: item["created_ms"].as_u64().unwrap_or_default(),
                    },
                ));
                if crate::slug_progress::progress(&staging) != item["progress"] {
                    return Err(format!(
                        "{archive} does not match the progress recorded in {FILE}"
                    ));
                }
            }
            Ok(())
        })();
        if let Err(error) = result {
            for (staging, _, _) in &staged {
                let _ = fs::remove_dir_all(staging);
            }
            return Err(error);
        }
        Ok(staged)
    }
}
