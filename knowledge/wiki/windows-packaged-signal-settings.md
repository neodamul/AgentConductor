# Windows managed-workspace package probe

## Issue #226 status

The current Rust #9 parent already contains the workspace restore/delete implementation and its local launch/registry regressions. The first-run UI may persist the requested workspace under AgentConductor's managed workspace root, so an exact-package UI probe must read `~/.agentconductor/workspaces.json` rather than assume the requested path is the active workspace.

## Package test contract

`tests/support/signal_workspace_registry.rs` waits up to 10 seconds for the exact canonical workspace to appear in the registry and reports the observed registry on timeout. `tests/windows_signal_settings.rs` then verifies persisted signal transport and port values at that resolved workspace. The helper regression covers exact-workspace selection and delayed first-run registry persistence; temporary test directories are removed on drop.

An earlier Windows package run [40805049482](https://github.com/neodamul/AgentConductor/actions/runs/40805049482) passed the signal-settings journey on an older #9 head. A later package run failed the separate shared `windows_tool_decision` journey. Neither proves the updated `#260` package. Keep bug #226 and #9 package acceptance open until the current Windows binary passes the restore and signal-settings journeys.

## Local verification

The local registry, signal-settings dispatch/window-refresh and source-layout suites pass 20 tests; workspace restore/delete suites pass 5/5. The Windows-specific exact-binary test is ignored on non-Windows hosts and requires the packaged Windows executable.

AgentMemory record: `mem_mvxyyn29_b1fa84d9dd35`.
