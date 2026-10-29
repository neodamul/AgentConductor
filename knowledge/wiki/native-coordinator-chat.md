# Native coordinator chat

The Workspace chat is an optional side panel, opened explicitly and held only for the current window. It asks the selected coordinator to answer questions about the project, its plans, and the offline manual. It does not persist conversation history.

The Rust UI sends the request through the existing Startup operation slot; planning and chat cannot occupy it at the same time. Requests and answers use bounded temporary artifacts, and consumed prompt/answer artifacts are removed. A new request cleans artifacts from a previous chat while preserving diagnostic logs for a failed run until cleanup.

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

The regression `tests/native_chat_unknown.rs` verifies accepted and rejected outcomes through egui's accessible UI tree. The focused test passes locally. Full offline suite and exact packaged macOS/Windows interaction remain pending; this feature does not increase the accepted workflow count until those gates pass.
