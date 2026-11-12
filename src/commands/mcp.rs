//! On-demand MCP worker commands; settings contain references, never resolved secrets.
pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    match args {
        [kind, root, name, rest @ ..]
            if (kind == "mcp-import" && rest.len() <= 1)
                || (kind == "mcp-discover" && rest.len() == 1) =>
        {
            Some((|| {
                let root = std::path::Path::new(root);
                let valid =
                    |n: &str| n.ends_with(".json") && !n.contains(['/', '\\']) && n.len() <= 100;
                if !valid(name)
                    || (kind == "mcp-import"
                        && rest
                            .first()
                            .is_some_and(|n| !valid(n) || !n.starts_with("preview-")))
                {
                    return Err("invalid MCP result name".into());
                }
                let saved: serde_json::Value =
                    serde_json::from_slice(&agentconductor::mcp::load(&root.join("startup.json"))?)
                        .map_err(|_| "invalid workspace settings")?;
                let settings=agentconductor::mcp::Settings::parse(&serde_json::json!({"mcp":saved["config"]["mcp"],"goal_sources":saved["config"]["goal_sources"]}).to_string())?;
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| e.to_string())?;
                let result = runtime.block_on(async {
                    if kind == "mcp-import" {
                        let profiles: Vec<agentconductor::plan::AgentProfile> =
                            serde_json::from_value(saved["config"]["profiles"].clone())
                                .map_err(|_| "invalid coordinator profiles")?;
                        let profile = agentconductor::mcp::coordinator_profile(
                            &profiles,
                            saved["config"]["coordinator"].as_str().unwrap_or(""),
                        )?;
                        // GitHub #903: progress-<id>.json beside preview-<id>.json; a retry keeps the previous result's read documents.
                        let mut progress = agentconductor::mcp::Progress::file(
                            root.join("mcp")
                                .join(name.replacen("preview-", "progress-", 1)),
                        );
                        let keep = rest.first().and_then(|previous| {
                            serde_json::from_slice(
                                &agentconductor::mcp::load(&root.join("mcp").join(previous))
                                    .ok()?,
                            )
                            .ok()
                        });
                        agentconductor::mcp::preview_with_progress(
                            &settings,
                            saved["config"]["goal"].as_str().unwrap_or(""),
                            root,
                            profile,
                            &mut progress,
                            keep,
                        )
                        .await
                        .and_then(|p| serde_json::to_value(p).map_err(|e| e.to_string()))
                    } else {
                        agentconductor::mcp::discover(&settings, &rest[0], root).await
                    }
                })?;
                agentconductor::mcp::store(
                    root,
                    name,
                    &serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?,
                )?;
                let mut audit = agentconductor::state::Store::open(root.join("state.sqlite"))?;
                for (status, items) in [
                    ("read", &result["records"]),
                    ("failed", &result["failures"]),
                ] {
                    for item in items.as_array().into_iter().flatten() {
                        let binding = settings
                            .goal_sources
                            .get(item["reference"]["kind"].as_str().unwrap_or(""));
                        audit.record("mcp_read_tool",&serde_json::json!({"reference":item["reference"],"server":item.get("server").cloned().or_else(||binding.map(|b|serde_json::json!(b.server))),"tool":item.get("tool").cloned().or_else(||binding.map(|b|serde_json::json!(b.tool))),"status":status,"sha256":item["sha256"]}),"Configured read-only MCP source tool result; no body or credentials")?;
                    }
                }
                audit.record("mcp_read_completed",&serde_json::json!({"kind":kind,"artifact":name,"records":result["records"].as_array().map_or(0,Vec::len),"failures":result["failures"].as_array().map_or(0,Vec::len)}),"Read-only MCP operation completed; no source body in audit")?;
                println!("{}", serde_json::json!({"artifact":name}));
                Ok(())
            })())
        }
        _ => None,
    }
}
