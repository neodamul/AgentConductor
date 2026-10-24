# Native coordinator chat

The Workspace chat is an optional side panel, opened explicitly and held only for the current window. It asks the selected coordinator to answer questions about the project, its plans, and the offline manual. It does not persist conversation history.

The Rust UI sends the request through the existing Startup operation slot; planning and chat cannot occupy it at the same time. Requests and answers use bounded temporary artifacts, and consumed prompt/answer artifacts are removed. A new request cleans artifacts from a previous chat while preserving diagnostic logs for a failed run until cleanup.

The coordinator invocation uses `coordinator::read_only_args`, which verifies the configured CLI profile's read-only mode and applies provider-specific restrictions. The prompt also tells the coordinator not to write files or run commands and to give the operator commands to run. This is a CLI subprocess using the profile's existing authentication, not an API-key service or a second polling process.

## Source map

- `src/native_ui_probe/workspace/chat.rs`: panel controls and current-window conversation state.
- `src/ui/setup/chat.rs`: bounded request artifact and Startup job submission/result consumption.
- `src/chat.rs`: prompt/answer bounds and read-only coordinator request.
- `src/coordinator.rs`: verified provider-specific read-only invocation.
- `tests/chat_prompt.rs`, `tests/native_workspace_chat.rs`, `tests/startup_chat.rs`: prompt, UI, and dispatch regressions.
- `docs/user-manual.md`, `docs/user-manual.ko.md`: operator-facing limits and usage.

Packaged macOS/Windows UI interaction remains unverified; this feature does not increase the accepted workflow count until the exact packages are reviewed.
