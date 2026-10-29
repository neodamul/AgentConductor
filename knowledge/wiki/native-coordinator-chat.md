# Native coordinator chat

The Workspace chat is an optional side panel that asks the selected coordinator to answer questions about the project, its plans, and the offline manual. The transcript is stored with the workspace's bounded chat artifacts, restored from the next snapshot after reopening, and includes a running chat job so the UI can resume following it. The retained transcript is capped at the latest 40 turns.

The Rust UI sends the request through the existing Startup operation slot; planning and chat cannot occupy it at the same time. Only the latest question crosses Startup IPC; the worker rebuilds earlier turns from the bounded transcript artifact stored with the workspace. The worker keeps that input artifact so a completed transcript can be restored after restart; the next accepted question or an explicit reset removes older chat artifacts. A workspace from an older build that has only an answer artifact remains readable, although its deleted question cannot be recovered. Prompt context clips older turns at a UTF-8 boundary and clips older answers instead of rejecting a follow-up because one answer exceeded the per-question bound.

The coordinator invocation uses `coordinator::read_only_args`, which verifies the configured CLI profile's read-only mode and applies provider-specific restrictions. The prompt also tells the coordinator not to write files or run commands and to give the operator commands to run. This is a CLI subprocess using the profile's existing authentication, not an API-key service or a second polling process.

If sending a chat request times out, the Rust UI treats acceptance as unknown. It keeps the message pending and disables another send while reading coordinator history through the normal workspace snapshot. A matching history entry follows the accepted job; unchanged history restores the original draft and timeout. A failed history read is retried without showing a false rejection. This prevents duplicate prompts when the coordinator accepted a request but its acknowledgement was lost.

## Source map

- `src/native_ui_probe/workspace/chat.rs`: panel controls and current-window conversation state.
- `src/native_ui_probe/workspace/chat/{outcome,reset,send}.rs`: distinguish unknown transport outcomes, reconcile the pending turn against history, and retry snapshots.
- `src/native_ui_probe/probe/refresh.rs`, `src/native_ui_probe/poll/errors.rs`, `src/native_ui_probe/app.rs`: request history while reconciling, retry failed snapshots, and schedule the reconciliation refresh.
- `src/ui/setup/chat.rs`: bounded request artifact and Startup job submission/result consumption.
- `src/chat.rs`: prompt/answer bounds and read-only coordinator request.
- `src/coordinator.rs`: verified provider-specific read-only invocation.
- `tests/chat_prompt.rs`, `tests/native_workspace_chat.rs`, `tests/startup_chat.rs`: prompt, UI, and dispatch regressions.
- `docs/user-manual.md`, `docs/user-manual.ko.md`: operator-facing limits and usage.

GitHub #189 / R-179 is still in progress: the native panel needs its terminal-style prompt, Enter/Shift+Enter, and question-history controls. Do not treat history persistence or prompt tests as packaged UI parity.

The regression `tests/native_chat_unknown.rs` verifies accepted and rejected outcomes through egui's accessible UI tree. The focused test passes locally. Full offline suite and exact packaged macOS/Windows interaction remain pending; this feature does not increase the accepted workflow count until those gates pass.
