//! On-demand goal optimizer worker (GitHub #180). The page writes `optimize-input-<id>.json`
//! into the private artifact folder; the proposal is stored as an immutable `optimize-<id>.json`.
pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    let [kind, root, id] = args else { return None };
    if kind != "optimize" {
        return None;
    }
    Some((|| {
        let root = std::path::Path::new(root);
        if id.is_empty()
            || id.len() > 60
            || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err("invalid optimize id".into());
        }
        let saved: serde_json::Value =
            serde_json::from_slice(&agentconductor::mcp::load(&root.join("startup.json"))?)
                .map_err(|_| "invalid workspace settings")?;
        let input_path = root.join("mcp").join(format!("optimize-input-{id}.json"));
        let bytes = agentconductor::mcp::load(&input_path)?;
        let _ = std::fs::remove_file(&input_path);
        let input: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| "invalid optimize input")?;
        let profiles: Vec<agentconductor::plan::AgentProfile> =
            serde_json::from_value(saved["config"]["profiles"].clone())
                .map_err(|_| "invalid coordinator profiles")?;
        let profile = agentconductor::mcp::coordinator_profile(
            &profiles,
            saved["config"]["coordinator"].as_str().unwrap_or(""),
        )?;
        let project =
            std::path::PathBuf::from(saved["config"]["repository"].as_str().unwrap_or(""));
        if !project.is_dir() {
            return Err("choose a project folder first".into());
        }
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        let run_dir = root.join(format!("optimize-run-{id}"));
        let result = runtime.block_on(agentconductor::prompt_optimizer::optimize(
            profile,
            &project,
            &run_dir,
            input["goal"].as_str().unwrap_or(""),
            input["language"].as_str().unwrap_or("English"),
        ));
        let _ = std::fs::remove_dir_all(&run_dir);
        let proposal = result?;
        agentconductor::mcp::store(
            root,
            &format!("optimize-{id}.json"),
            &serde_json::to_vec(&proposal).map_err(|e| e.to_string())?,
        )?;
        println!(
            "{}",
            serde_json::json!({"artifact": format!("optimize-{id}.json")})
        );
        Ok(())
    })())
}
