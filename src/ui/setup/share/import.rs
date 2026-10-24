use super::super::*;
use super::{FILE, LOCAL_CLAIMS, folder, read_shared};

type StagedSlug = (PathBuf, PathBuf, crate::slugs::Slug);

impl Startup {
    pub(in crate::ui::setup) fn import_settings(
        &self,
        saved: &mut Saved,
        dir: &Path,
        keep_verified: bool,
    ) -> Result<serde_json::Value, String> {
        let value = read_shared(dir)?;
        let mut profiles: Vec<AgentProfile> = serde_json::from_value(value["agents"].clone())
            .map_err(|e| {
                crate::ui::messages::render("native.invalid_agents", &[("e", &e.to_string())])
            })?;
        super::super::profiles::derive_permissions(&mut profiles);
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
                crate::ui::messages::render("native.invalid_roles", &[("e", &e.to_string())])
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
                    &[("e", &e.to_string())],
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
                        &[("e", &e.to_string())],
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
                        &[("archive", archive), ("slug", slug)],
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
                        last_run: None,
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
