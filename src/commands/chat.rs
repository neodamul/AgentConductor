//! On-demand coordinator chat worker for the workspace window.
pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    let [kind, root, id] = args else { return None };
    if kind != "chat" {
        return None;
    }
    Some((|| {
        if id.is_empty()
            || id.len() > 60
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err("invalid chat id".into());
        }
        let root = std::path::Path::new(root);
        let input_name = format!("chat-input-{id}.json");
        let bytes = agentconductor::mcp::load(&root.join("mcp").join(&input_name))?;
        let input: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| "invalid chat input")?;
        let turns: Vec<agentconductor::chat::Turn> =
            serde_json::from_value(input["turns"].clone()).map_err(|_| "invalid chat turns")?;
        let saved: serde_json::Value =
            serde_json::from_slice(&agentconductor::mcp::load(&root.join("startup.json"))?)
                .map_err(|_| "invalid workspace settings")?;
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
            .map_err(|error| error.to_string())?;
        let run_dir = root.join(format!("chat-run-{id}"));
        let answer = runtime.block_on(agentconductor::chat::ask_in(
            profile,
            &project,
            &run_dir,
            &turns,
            input["language"].as_str().unwrap_or("English"),
            input["workspace"].as_str().unwrap_or(""),
        ));
        let answer = match answer {
            Ok(answer) => {
                let _ = std::fs::remove_dir_all(&run_dir);
                answer
            }
            Err(error) => return Err(error),
        };
        agentconductor::mcp::store(
            root,
            &format!("chat-{id}.json"),
            &serde_json::to_vec(&serde_json::json!({"answer": answer}))
                .map_err(|error| error.to_string())?,
        )?;
        println!("{{\"artifact\":\"chat-{id}.json\"}}");
        Ok(())
    })())
}
