# Native settings dialog hierarchy

## Latest visual review — v0.1.36 package / 2026-10-23

In the existing single macOS package window at 1140×768, the workspace menu overlays the workspace page without reflow. Settings → MCP registration and Settings → Workspace settings sharing remain legible in Korean dark theme with no visible text overlap. No settings or workspace actions were saved. This is component-level review only: the screenshot was not retained because the saved user workspace was open, and no packaged workflow acceptance is added. Capture release-manual images from an isolated synthetic workspace.

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
save action, and sequential close behavior. The exact v0.1.36 macOS package from source
`cff95dd02677519ebbb7c359f5ca9e3da0c504ed` was reopened in one app window. Korean/dark showed
“설정 · MCP 서버” above “MCP 서버 등록”; English/dark and English/light showed “SETTINGS · MCP
SERVERS” above “Register MCP servers”. The saved workspace was not modified and no provider was
called. Evidence: `docs/evidence/issue9-cff95dd-mcp-dialog.json`. The screenshot was inspected
in-session but not retained; this component check does not count as a packaged workflow acceptance.
