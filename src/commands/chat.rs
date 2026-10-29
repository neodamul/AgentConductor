//! On-demand coordinator chat worker (GitHub #115). The page writes `chat-input-<id>.json`
//! into the private artifact folder; the answer is stored as an immutable `chat-<id>.json`.
pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    let [kind, root, id] = args else { return None };
    if kind != "chat" {
        return None;
    }
    Some((|| {
        let root = std::path::Path::new(root);
        if id.is_empty()
            || id.len() > 60
            || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err("invalid chat id".into());
        }
        let saved: serde_json::Value =
            serde_json::from_slice(&agentconductor::mcp::load(&root.join("startup.json"))?)
                .map_err(|_| "invalid workspace settings")?;
        let input: serde_json::Value = serde_json::from_slice(&agentconductor::mcp::load(
            &root.join("mcp").join(format!("chat-input-{id}.json")),
        )?)
        .map_err(|_| "invalid chat input")?;
        let turns: Vec<agentconductor::chat::Turn> =
            serde_json::from_value(input["turns"].clone()).map_err(|_| "invalid chat turns")?;
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
        let answer = runtime.block_on(agentconductor::chat::ask_in(
            profile,
            &project,
            &root.join(format!("chat-run-{id}")),
            &turns,
            input["language"].as_str().unwrap_or("English"),
            &agentconductor::chat_context::workspace(&saved, &project),
        ))?;
        agentconductor::mcp::store(
            root,
            &format!("chat-{id}.json"),
            &serde_json::to_vec(&serde_json::json!({"answer": answer}))
                .map_err(|e| e.to_string())?,
        )?;
        println!(
            "{}",
            serde_json::json!({"artifact": format!("chat-{id}.json")})
        );
        Ok(())
    })())
}
