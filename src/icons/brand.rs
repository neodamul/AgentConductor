//! GitHub #406 / DD-222: pinned color brand marks for Claude Code and Codex (MIT, Lobe Icons;
//! see third_party/lobe-icons/SOURCE.md). The files are embedded unmodified.
use crate::protocol::Provider;

const CLAUDE_CODE: &str = include_str!("../../third_party/lobe-icons/claude-code-color.svg");
const CODEX: &str = include_str!("../../third_party/lobe-icons/codex-color.svg");

pub fn brand_icon(provider: Provider) -> Option<String> {
    let svg = match provider {
        Provider::Claude => CLAUDE_CODE,
        Provider::Codex => CODEX,
        Provider::OpenCode | Provider::Custom => return None,
    };
    Some(format!(
        "data:image/svg+xml;base64,{}",
        super::base64(svg.as_bytes())
    ))
}

/// Inert templates contain only the two pinned repository assets, never configuration data.
pub(crate) fn templates() -> String {
    format!(
        "<template id=\"ac-brand-claude\">{CLAUDE_CODE}</template><template id=\"ac-brand-codex\">{CODEX}</template>"
    )
}
