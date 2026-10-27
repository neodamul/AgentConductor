# Native coordinator chat

The Workspace chat is an optional side panel, opened explicitly. Its transcript is stored in the local workspace startup state under a canonical repository-path key, restored from snapshots, and retained when the panel or app closes or the operator switches workspaces. A running chat job is reconnected through the existing Startup job snapshot. The selected coordinator answers questions about the project, its plans, and the offline manual.

The Rust UI sends only the newest question through the existing Startup operation slot; the controller rebuilds the CLI prompt from up to 40 stored turns. Oversized turn text is clipped and total prompt size stays bounded. Planning and chat cannot occupy the slot at the same time. The request and answer use bounded temporary artifacts. New conversation clears transcript state but retains diagnostic run folders. The egui view uses monospace transcript text, Enter to send, Shift+Enter for a newline, and Up to recall the prior operator question.

The coordinator invocation uses `coordinator::read_only_args`, which verifies the configured CLI profile's read-only mode and applies provider-specific restrictions. The prompt also tells the coordinator not to write files or run commands and to give the operator commands to run. This is a CLI subprocess using the profile's existing authentication, not an API-key service or a second polling process.

## Source map

- `src/native_ui_probe/workspace/chat.rs`: panel controls and restored conversation state.
- `src/ui/setup/chat.rs`: bounded request artifact and Startup job submission/result consumption.
- `src/chat.rs`: 40-turn prompt/answer bounds and read-only coordinator request.
- `src/coordinator.rs`: verified provider-specific read-only invocation.
- `tests/chat_prompt.rs`, `tests/native_workspace_chat.rs`, `tests/startup_chat.rs`: prompt, UI, and dispatch regressions.

Graphify could not answer a scoped code query because this worktree has no `graphify-out/graph.json`; no graph was generated to avoid a repo-wide artifact. CocoIndex is unavailable because no `cocoindex` executable is installed in this environment.
- `docs/user-manual.md`, `docs/user-manual.ko.md`: operator-facing limits and usage.

Packaged macOS/Windows UI interaction remains unverified; this feature does not increase the accepted workflow count until the exact packages are reviewed.
