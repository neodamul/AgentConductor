//! GitHub #585: a `claude` started for the usage lookup must not inherit the Claude Code session that started this app.
//! The session markers make it report API billing or "not logged in"; user auth, keys and provider settings stay.

/// True for the variables a Claude Code session exports about itself (not credentials or provider settings).
#[cfg_attr(not(any(target_os = "macos", windows)), allow(dead_code))]
pub(super) fn is_session_marker(name: &str) -> bool {
    let name = name.to_ascii_uppercase();
    name == "CLAUDECODE"
        || name.starts_with("CLAUDE_AGENT_SDK")
        || (name.starts_with("CLAUDE_CODE_")
            && !["TOKEN", "KEY", "USE_", "PROXY", "CERT", "CONFIG"]
                .iter()
                .any(|kept| name.contains(kept)))
}
