# Native settings dialog hierarchy

## Contract

Native settings windows keep the window title distinct from the content heading in English and
Korean. Inside the window, render the localized Settings section as a compact eyebrow, followed by
the task-specific heading. Keep an in-content localized close action for keyboard and screen-reader
access; the operating-system close control alone is insufficient.

## MCP registration

The existing dialog in `src/ui/mcp.html` uses the `SETTINGS · MCP SERVERS` eyebrow and the
`Register MCP servers` heading. The Rust `native_ui_probe::workspace::mcp::State` follows the same
hierarchy and names its window “MCP server settings” / “MCP 서버 설정”; the content heading remains
“Register MCP servers” / “MCP 서버 등록”. The OpenCode JSON/JSONC editor, asynchronous save,
explicit tool discovery, and close action are unchanged.

## Verification

`tests/native_ui_mcp.rs` checks the rendered section label and distinct window/content title in both
languages. `tests/native_ui_settings_journey.rs` checks the settings dialog still exposes its title,
save action, and sequential close behavior. The exact packaged window has not yet been rebuilt and
visually reopened for this change; do not count it as a packaged workflow acceptance.
