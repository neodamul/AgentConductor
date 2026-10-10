# Live tool decisions

Trace: R-30 → BG-13 → SF-25 → UC-22 → QAS-23 → DDR-01/DDR-09 → DD-32.

## Problem

Stop-and-resume tool approval bound the exact tool input, but a resumed Claude attempt
regenerates its Bash command or description, so approvals rarely matched and native runs
looped in `waiting_tool_approval` (observed in native app acceptance on 2026-10-10).

## Contract

The guarded worker keeps the PreToolUse callback open while a pending ledger request
awaits a human. Hook timeout and control-evaluation limit follow the decision window
(`AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS`, default 600, maximum 3000, 0 = previous
stop-and-resume). While the run is `running`, `tool-approve`/`tool-reject` change only the
transactional ledger; the coordinator keeps the run lock and report. A fingerprint-only
workspace change re-keys the identical request at decision time. Consumption re-verifies
task, session, plan, epoch and workspace path. Rejection is a per-call deny. Stop or an
unanswered window falls back to the stopped/waiting flow without executing the command.

## Evidence

Red: both live tests failed (decision refused while running). Green after holding the
callback, raising the 30-second control-evaluation cap and binding the workspace
fingerprint at decision time (parallel mock writers changed it within 250 ms).
Full regression: 203 passed, 0 failed, 8 ignored. Clippy, fmt and whitespace pass.

## Source fingerprints

| Source | SHA-256 |
| --- | --- |
| `src/native_guard.rs` | `c409b430ccce84d52942ecc5ac8992d2034b193bf4fe9b16cea9ae35d2f9af93` |
| `src/process.rs` | `9949d1f354d1f746bc906d4437d1cda94c7c946c7423ed7736b7a72215ad5a57` |
| `src/workflow.rs` | `ae12091ab6ef39d469bda55251e5e8688b8e42cc7ceb9648408599ba24b67fc5` |
| `src/claude_control.rs` | `3e4347847f1fc374c0e90619be54fea3d2a6e54d1db061596ca355ebad8cdc79` |
| `src/ui/room.js` | `01a05cc0448aa65c99f7acba5e67c539340abdd203d91b0fae852b8bf65d1cd7` |
| `tests/live_tool_decision.rs` | `a91b2e1ef01e9a57db04e6e316b426d09df0895cafa833b9058cd302fdc7c23c` |
