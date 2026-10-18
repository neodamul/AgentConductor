use super::super::*;
use super::{FILE, FORMAT, folder};

impl Startup {
    pub(in crate::ui::setup) fn export_settings(
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
                    &[("value1", &entry.slug.to_string())],
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
}
