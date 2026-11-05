use super::*;

pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    match args {
        [command, profiles_path, agent_id, dir] if command == "models-discover" => Some((|| {
            let mut profiles: Vec<AgentProfile> =
                serde_json::from_slice(&std::fs::read(profiles_path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let agent = profiles
                .iter_mut()
                .find(|a| a.id == *agent_id)
                .ok_or("unknown agent")?;
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let catalog =
                runtime.block_on(agentconductor::models::discover(agent, Path::new(dir)))?;
            println!(
                "{}",
                serde_json::to_string(&catalog).map_err(|e| e.to_string())?
            );
            agent.catalog = Some(catalog);
            write_profiles(profiles_path, &profiles)
        })(
        )),
        [command, profiles_path, agent_id, catalog_path] if command == "catalog-import" => {
            Some((|| {
                let mut profiles: Vec<AgentProfile> = serde_json::from_slice(
                    &std::fs::read(profiles_path).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                let catalog: agentconductor::plan::ModelCatalog = serde_json::from_slice(
                    &std::fs::read(catalog_path).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                if catalog.models.is_empty()
                    || catalog.queried_at <= 0
                    || catalog.source.trim().is_empty()
                {
                    return Err("catalog needs observed models and query provenance".into());
                }
                let agent = profiles
                    .iter_mut()
                    .find(|a| a.id == *agent_id)
                    .ok_or("unknown agent")?;
                agent.catalog = Some(catalog);
                for model in &agent.catalog.as_ref().unwrap().models {
                    if model.is_empty()
                        || model.starts_with('-')
                        || model.chars().any(char::is_whitespace)
                    {
                        return Err("invalid model ID".into());
                    }
                }
                write_profiles(profiles_path, &profiles)
            })())
        }
        _ => None,
    }
}
