# Workspace keyboard acceptance record

Date: 2026-10-18
Binary: `aarch64-apple-darwin`, SHA-256 `c4618e60f777de0f96e21c69864fbe34267686a70f771eebf435eb921cc36d7f`
Source code: current `codex/issue-9-native-ui-continuation` HEAD; the executable matches the exact-source local package.

## Evidence

- Launched a single preview app with an isolated temporary home and workspace; no saved workspace, profile, or user setting was changed.
- Tab reached Workspace menu, Settings, agent registration, stage navigation, name, project folder, folder chooser, goal, Jira/Confluence references, execution isolation, save, and settings import/export in order.
- Disabled coordinator and import-preview controls were skipped.
- Space selected Git worktree; the selection was returned to Shared and never saved.
- AccessKit exposed labels for the fields, navigation, radios, and actions. Full screen-reader acceptance was not attempted.
- Visually inspected the CUA screenshot. Saving a display capture with macOS `screencapture` failed with `could not create image from display`; no screenshot asset was replaced.

## Scope boundary

This verifies keyboard traversal on the clean Workspace screen only. It does not verify Team, Settings, Review Plan, Agent Room, keyboard activation for every action, screen readers, Korean IME, Windows, or screenshot provenance for public documentation. Keep issue #9 open and the 0/25 end-to-end workflow count unchanged.
