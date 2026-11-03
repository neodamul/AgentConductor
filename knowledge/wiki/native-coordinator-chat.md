# Native coordinator chat

The Workspace chat is an optional side panel that asks the selected coordinator to answer questions about the project, its plans, and the offline manual. The transcript is stored with the workspace's bounded chat artifacts, restored from the next snapshot after reopening, and includes a running chat job so the UI can resume following it. The retained transcript is capped at the latest 40 turns.

The Rust UI sends the request through the existing Startup operation slot; planning and chat cannot occupy it at the same time. Only the latest question crosses Startup IPC; the worker rebuilds earlier turns from the bounded transcript artifact stored with the workspace. The worker keeps that input artifact so a completed transcript can be restored after restart; the next accepted question or an explicit reset removes older chat artifacts. A workspace from an older build that has only an answer artifact remains readable, although its deleted question cannot be recovered. Prompt context clips older turns at a UTF-8 boundary and clips older answers instead of rejecting a follow-up because one answer exceeded the per-question bound.

The coordinator invocation uses `coordinator::read_only_args`, which verifies the configured CLI profile's read-only mode and applies provider-specific restrictions. The Workspace setup screen lets the operator choose a model from the coordinator profile's observed catalog and shows a localized read-only compatibility warning inline before chat is opened. The prompt also tells the coordinator not to write files or run commands and to give the operator commands to run. This is a CLI subprocess using the profile's existing authentication, not an API-key service or a second polling process.

The Workspace warning is evaluated against the currently selected profile, not a stale snapshot. A profile change therefore updates the reason before chat opens. Model selection is limited to values in that profile's observed `catalog.models`; choosing the CLI default removes the optional `coordinator_model` override, while a catalog selection persists through the existing workspace save request.

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

GitHub #189 / R-179 remains in progress. On root PR #22 code head `d5731f11`, the current chat/accessibility/Windows-driver/package-diagnostic/source-layout suites pass 23 tests across seven targets, and warning-denied all-target Clippy passes. Platform Validation run `41500210315` was cancelled; current run `41553629403` covers cross-platform build/native validation but not the exact packaged chat journeys. The UTM Z: share contains a Windows package from `ac3f8b32` (2026-10-26), before the accessible-editor fix `fbc2280a` (2026-10-30); its smoke evidence covers CLI merge cases only, so it is stale and not chat UI evidence. The guest has no Cargo or Rust compiler. The terminal-style transcript and keyboard controls pass local egui interaction tests. Do not treat source tests, stale packages, or a host Job Object skip as packaged UI parity.

The regression `tests/native_chat_unknown.rs` verifies accepted and rejected outcomes through egui's accessible UI tree. The five focused suites pass 22 tests; warning-denied all-target Clippy, formatting, diff checks, and the 300-line limit pass locally. Full all-target tests remain unverified: a cold run was interrupted after 130 tests across 24 suites, and a cached retest exited 130 after 58 tests across 5 suites. Exact packaged macOS/Windows interaction remains pending; this feature does not increase the accepted workflow count until those gates pass.

## Desktop-close lifecycle (2026-11-01)

The coordinator still runs as one read-only CLI subprocess per question. Only that `chat` operation is launched independently of the desktop process so a delayed answer can finish after the window closes. On Windows it requests Job Object breakaway, a separate process group, and no console window. If the host denies breakaway with access denied, Rust checks whether the desktop process belongs to any Windows Job Object. It retries as a hidden child only when the process is outside a host job. If membership is true or cannot be queried, the job is persisted as `not_spawned` and the UI receives an explicit error; it must never report the chat as running when survival after app close is uncertain. Other UI actions retain their current owned-process behavior. The worker writes its existing bounded transcript/input/result artifacts; reopening the workspace follows the persisted job and reads the final answer. The `chat` command skips the messenger shutdown flush so completion is not delayed.

The current request-pool lanes must be preserved. PR #192's old `sync_channel(1)` worker implementation is not safe to merge onto the current event-driven UI base. The Windows package harness now covers transcript restore, reconnect after closing the app during a running chat, and late-answer recovery. Local macOS verification passes; exact Windows packaged lifecycle acceptance remains open, so issue #189 and #9 remain open.

## Windows package UI Automation selection (2026-11-01)

Run [41270341316](https://github.com/neodamul/AgentConductor/actions/runs/41270341316) passed validation and the macOS package but failed the Windows stop/resume package journey because its driver could not find the `Stop reason` editor after invoking `Stop run`. The previous driver searched all top-level windows owned by the process, including hidden/offscreen windows. It now filters top-level windows and descendants using `IsOffscreen` and a non-empty `BoundingRectangle`, and prints matching process-window diagnostics on timeout. `lifecycle_driver_exposes_stop_resume_and_recovery_observations` was run red then green locally. The updated exact Windows package journey still needs to pass before #9 acceptance changes.

The native multiline chat editor also exposes its localized placeholder as its AccessKit accessible name. `tests/native_workspace_chat_accessibility.rs` guards this contract for Windows UI Automation; it does not replace exact packaged Windows verification.

## Resumed chat UI acceptance and PR #255 audit (2026-11-01)

The current exact-package Windows reconnect scenario also asserts the accessible editor remains disabled while the persisted chat job is running and becomes enabled after the answer is visible. This assertion lives on the active #9 parent branch so it can be exercised with the current package. PR #255's separate 500 ms status poll was not ported: QAS-192/DD-206 deliberately rely on scoped events and an independent fixed 30-second lost-event recovery check to avoid frequent snapshot polling. Issue #189 stays open until the strengthened scenario passes against the exact Windows package.

## Slow repository lookup regression and PR #239 audit (2026-11-02)

`tests/startup_chat.rs::chat_submission_does_not_wait_for_repository_head` wraps Git so `rev-parse HEAD` sleeps for 20 seconds, then asserts the Rust chat dispatch finishes in under two seconds. The helper scopes the environment override with a restoring lock and removes its temporary repository on drop. The first port exposed an old `turns` request shape and a repository revision token where the current Rust protocol requires `text` and `chat_revision`; the updated test passes in 0.21 seconds. PR #239 was closed as superseded after this regression was preserved on the active #260 parent. The product behavior was already present there; no old source implementation was copied.

## Current-head accessibility verification (2026-11-03)

At root PR #22 head `397069af`, the coordinator chat editor's localized placeholder is its AccessKit name. The Windows UI Automation driver filters to a visible, enabled multiline edit with that localized name. Local verification passes 11 tests across the AccessKit rendering, Windows script contract, worker lifecycle, transcript interaction, and source-layout suites. The exact Windows package remains the acceptance boundary: if the host Job Object prevents the chat worker from surviving app close, the test reports an explicit skip and must not be counted as a pass.

# Current verification checkpoint — 2026-11-03

At Rust branch head `ef0bc07fcfe670019b939ce13181ad64c1a9e07c`, the complete local macOS command `cargo test --locked --offline --all-targets` passed 1,314 tests with 16 ignored across 409 suites (283.30 seconds). This is source-level regression evidence only. It does not prove exact packaged Windows transcript persistence, reconnect, or interactive UI automation; retain those as explicit package acceptance gates.
