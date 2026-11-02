# Closed PR #255 Regression Audit

Date: 2026-11-03
Repository branch: `codex/issue-9-native-ui-parity`

## Scope

Compare the closed PR #255 event-driven native UI, worker concurrency, coordinator chat persistence, and package-driver claims against the current issue #9 candidate. PR #255 targeted the moving issue #9 branch and is closed without merge. Several module and test paths were consolidated after the PR was authored.

## Current regression coverage

- `tests/native_ui_probe/events.rs`: event fanout, coalescing, disconnected listeners, source suppression, persisted state observation, and workspace-switch listener behavior.
- `tests/native_ui_probe/worker_concurrency.rs`: bounded parallel reads, independent work while refresh or external IO is blocked, serialized state changes, and stale reply rejection.
- `tests/native_ui_probe/poll_events.rs` and `poll_background_pending.rs`: asynchronous event/recovery refresh and preservation of pending operations and unsaved drafts.
- `tests/startup_chat.rs` plus `tests/support/startup_chat_timeout.rs`: chat history persistence and chat revision/timeout regression.
- `tests/launch_workspace.rs` and `tests/windows_workspace_reopen.rs`: saved-workspace launch and packaged Windows reopen behavior.

## Verification

Command:

```text
cargo test --locked --offline --features native-ui-probe --test native_ui_probe --test ui --test startup_chat --test windows_coordinator_chat --test launch_workspace --test native_ui_static_labels
```

Result: 27 passed across 6 test targets.

## Limits

This run verifies local source-level regression targets. Historical package evidence from the closed PR is not present in the current branch. No exact packaged Windows/macOS validation was performed in this audit. It does not advance #9 acceptance: 22/50 overall, packaged workflows 22/25, shipped/default Rust UI parity 0/25. Root PR #22 remains Draft.
