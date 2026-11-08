//! GitHub #585: a `claude` started for the usage lookup must not inherit the Claude Code session that started this app.
//! The session markers make it report API billing or "not logged in"; user auth, keys and provider settings stay.

/// True for the variables a Claude Code session exports about itself (not credentials or provider settings).
#[cfg_attr(not(any(target_os = "macos", windows)), allow(dead_code))]
pub(in crate::quota::terminal) fn is_session_marker(name: &str) -> bool {
    let name = name.to_ascii_uppercase();
    name == "CLAUDECODE"
        || name.starts_with("CLAUDE_AGENT_SDK")
        || (name.starts_with("CLAUDE_CODE_")
            && !["TOKEN", "KEY", "USE_", "PROXY", "CERT", "CONFIG"]
                .iter()
                .any(|kept| name.contains(kept)))
}

/// Removes the session markers from the environment a collector child will inherit.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(in crate::quota::terminal) fn scrub(
    command: &mut tokio::process::Command,
) -> &mut tokio::process::Command {
    for (name, _) in std::env::vars().filter(|(name, _)| is_session_marker(name)) {
        command.env_remove(name);
    }
    command
}
