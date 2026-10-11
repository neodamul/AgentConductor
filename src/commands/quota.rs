use super::*;
pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    match args {
        [command, dir, agent] if command == "workspace-quota" => Some((|| {
            let root = Path::new(dir).canonicalize().map_err(|e| e.to_string())?;
            let bytes = std::fs::read(root.join("startup.json")).map_err(|e| e.to_string())?;
            if bytes.len() > 1024 * 1024 {
                return Err("workspace settings exceed quota query bound".into());
            }
            let saved: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            let config: agentconductor::ui::setup::Config =
                serde_json::from_value(saved["config"].clone()).map_err(|e| e.to_string())?;
            let profile = config
                .profiles
                .iter()
                .find(|p| p.id == *agent)
                .ok_or("unknown quota agent")?;
            let query = root.join(format!(
                "quota-query-{}",
                agentconductor::native_guard::session_id()?
            ));
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let result = if profile.parser == Provider::Claude {
                runtime.block_on(agentconductor::quota::query_claude(
                    profile,
                    &query,
                    &config.repository,
                ))
            } else {
                runtime.block_on(agentconductor::quota::query(profile, &query))
            };
            let mut store = Store::open(root.join("quota.sqlite"))?;
            agentconductor::quota::persist(&mut store, profile, result)
        })()),
        _ => None,
    }
}
