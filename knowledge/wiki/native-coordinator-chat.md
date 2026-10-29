# Native coordinator chat

The Workspace chat is an optional side panel that asks the selected coordinator to answer questions about the project, its plans, and the offline manual. The transcript is stored with the workspace's bounded chat artifacts, restored from the next snapshot after reopening, and includes a running chat job so the UI can resume following it. The retained transcript is capped at the latest 40 turns.

The Rust UI sends the request through the existing Startup operation slot; planning and chat cannot occupy it at the same time. Only the latest question crosses Startup IPC; the worker rebuilds earlier turns from the bounded transcript artifact stored with the workspace. The worker keeps that input artifact so a completed transcript can be restored after restart; the next accepted question or an explicit reset removes older chat artifacts. A workspace from an older build that has only an answer artifact remains readable, although its deleted question cannot be recovered. Prompt context clips older turns at a UTF-8 boundary and clips older answers instead of rejecting a follow-up because one answer exceeded the per-question bound.

The coordinator invocation uses `coordinator::read_only_args`, which verifies the configured CLI profile's read-only mode and applies provider-specific restrictions. The prompt also tells the coordinator not to write files or run commands and to give the operator commands to run. This is a CLI subprocess using the profile's existing authentication, not an API-key service or a second polling process.

If sending a chat request times out, the Rust UI treats acceptance as unknown. It keeps the message pending and disables another send while reading coordinator history through the normal workspace snapshot. A matching history entry follows the accepted job; unchanged history restores the original draft and timeout. A failed history read is retried without showing a false rejection. This prevents duplicate prompts when the coordinator accepted a request but its acknowledgement was lost.

## Source map

- `src/native_ui_probe/workspace/chat.rs`, `src/native_ui_probe/workspace/chat/view.rs`: panel state and terminal-style transcript/input. Questions and answers use monospace `> ` / `< ` markers; Enter sends, Shift+Enter inserts a newline at the cursor, and Up/Down recalls operator questions while preserving an unsent draft.
- `src/native_ui_probe/workspace/chat/{outcome,reset,send}.rs`: distinguish unknown transport outcomes, reconcile the pending turn against history, and retry snapshots.
- `src/native_ui_probe/probe/refresh.rs`, `src/native_ui_probe/poll/errors.rs`, `src/native_ui_probe/app.rs`: request history while reconciling, retry failed snapshots, and schedule the reconciliation refresh.
- `src/ui/setup/chat.rs`: bounded request artifact and Startup job submission/result consumption.
- `src/chat.rs`: prompt/answer bounds and read-only coordinator request.
- `src/coordinator.rs`: verified provider-specific read-only invocation.
- `tests/chat_prompt.rs`, `tests/native_workspace_chat.rs`, `tests/native_chat_terminal.rs`, `tests/startup_chat.rs`, `tests/chat_startup_context.rs`: prompt, keyboard, UI, restart, and dispatch regressions.
- `docs/user-manual.md`, `docs/user-manual.ko.md`: operator-facing limits and usage.

GitHub #189 / R-179 remains in progress. The terminal-style transcript and keyboard controls now pass local egui interaction tests. Do not treat source tests as packaged UI parity.

The regression `tests/native_chat_unknown.rs` verifies accepted and rejected outcomes through egui's accessible UI tree. The five focused suites pass 22 tests; warning-denied all-target Clippy, formatting, diff checks, and the 300-line limit pass locally. Full all-target tests remain unverified: a cold run was interrupted after 130 tests across 24 suites, and a cached retest exited 130 after 58 tests across 5 suites. Exact packaged macOS/Windows interaction remains pending; this feature does not increase the accepted workflow count until those gates pass.

## Current-head packaged regression

The current #9 implementation stores the result pointer as `chat_result` in `startup.json`; bounded turns live in `mcp/chat-input-<id>.json`, and the answer is written to `mcp/chat-<id>.json`. The Windows exact-package UI regression uses a deterministic Rust coordinator fixture to verify transcript restoration after reopen, reconnect to an in-flight worker, and a 20-second answer while the chat stays open. The package workflow runs these window-driving cases serially. `AC_CHAT_WAIT=0` only exits the test driver after sending and does not reproduce the legacy 15-second UI request timeout. Local artifact reconstruction and driver-contract tests pass; exact Windows package interaction on the current head is still required before acceptance.

The earlier PR #192 package run is not current-head evidence: although its three chat journeys passed, the overall Windows package failed later in the tool-decision journey, and that branch used a different transcript persistence shape.

## Windows chat editor accessibility — 2026-10-30

Run `40955925655` failed all three exact Windows package coordinator-chat journeys before question submission. The UI Automation diagnostics exposed a disabled `Send` button but no edit matching the editor prompt. A local AccessKit regression reproduced the contract gap: the multiline editor had visible placeholder text but no accessible name. The Rust editor now sets its localized placeholder as its accessible name while retaining the same visible hint. The new regression and related chat, Windows driver-contract, and source-layout tests pass 11/11 across four suites; warning-denied Clippy, rustfmt, diff checks, and the 300-line source/test limit pass. This does not prove the corresponding Windows UI Automation mapping. Port the source change onto PR #255, rerun its exact package, and confirm all chat journeys before crediting workflow 23 or continuing to tool-decision acceptance.
