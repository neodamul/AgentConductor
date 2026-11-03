# Issue #9: exact-package UI audit

Date: 2026-10-20
Source: `7419557168ddc3ffe76c446cf292152cc9fc1932`
Package: macOS `aarch64-apple-darwin`, binary SHA-256 `6988a2dd852b23665be6f43f4d2d7e20092dbef5e20db49574cd6d1982aa6836`

## Setup

Closed the prior v0.1.31 app and opened the exact Rust package once with an isolated `HOME` and disposable workspace under `/private/tmp/agentconductor-issue9-ui-checkpoint-20261020`. No user workspace was opened or changed. The final single app window remains on the disposable workspace for review.

## Batched screens and interactions

- Visually inspected Workspace settings, the Settings menu, MCP server registration, custom profile registration, and Agent Roles in one window.
- Registered a local custom profile and assigned it to development, testing, and documentation roles.
- Created two saved workspaces, verified the menu listed the current workspace, and archived the inactive workspace.
- Saved a shared-folder test workspace, then launched a temporary Rust mock coordinator that reads stdin, delays 15 seconds, and returns three disjoint mock tasks. No provider CLI, API, network service, or worker task ran.
- While planning was active, the UI exposed `Stop planning and delete`. After confirmation, the coordinator returned from Running to Ready and the active workspace was archived into a fresh workspace. The deleted test workspace no longer appeared in the saved-workspace list.

## Result and limits

No new UI defect was reproduced in these interactions, and the packaged stop-and-delete path worked. The package UI screens were visually inspected, but screenshot bytes were not persisted as a public screenshot asset. This is component/interaction evidence, not a complete end-to-end workflow; conservative packaged UI acceptance remains 2/25. Windows, authenticated provider use, exact-head CI, full screen parity, accessibility/IME, and idle resource checks remain open.

Machine-readable evidence: `docs/evidence/issue9-7419557-packaged-ui-audit.json`.
