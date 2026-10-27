# Native coordinator chat

The Workspace chat is an optional side panel, opened explicitly. Its transcript is stored in the local workspace startup state under a canonical repository-path key, or under a deterministic folderless-workspace key derived from the persisted workspace root. The root path is the stable workspace identity; the display name and goal are mutable fields and are not identity keys. It is restored from snapshots and retained when the panel or app closes or the operator switches workspaces. A running chat job is reconnected through the existing Startup job snapshot. The selected coordinator answers questions about the project, its plans, and the offline manual.

The Rust UI sends only the newest question through the existing Startup operation slot; the controller stores and rebuilds the CLI prompt from the latest 40 turns. Oversized turn text is clipped and total prompt size stays bounded. Planning and chat cannot occupy the slot at the same time. The request and answer use bounded temporary artifacts. On app reopen, the first ordinary workspace snapshot consumes a completed-but-unread answer, persists it with the transcript, and clears its result reference; this covers a coordinator process that finished while the window was closed. New conversation clears transcript state but retains diagnostic run folders. The egui view uses monospace transcript text, Enter sends only while the editor has focus, Shift+Enter inserts a newline, and Up recalls the prior operator question.

The coordinator invocation uses `coordinator::read_only_args`, which verifies the configured CLI profile's read-only mode and applies provider-specific restrictions. The prompt also tells the coordinator not to write files or run commands and to give the operator commands to run. This is a CLI subprocess using the profile's existing authentication, not an API-key service or a second polling process.

## Source map

- `src/native_ui_probe/workspace/chat.rs`: panel controls and restored conversation state.
- `src/ui/setup/chat.rs`, `src/ui/setup/snapshot.rs`: bounded request, Startup result consumption, and recovery of completed responses on initial snapshots.
- `src/chat.rs`: 40-turn prompt/answer bounds and read-only coordinator request.
- `src/coordinator.rs`: verified provider-specific read-only invocation.
- `tests/chat_prompt.rs`, `tests/native_workspace_chat.rs`, `tests/native_workspace_chat_interaction.rs`, `tests/startup_chat.rs`, `tests/startup_chat_persistence.rs`: prompt, UI, dispatch, workspace-isolation, and persistence regressions.

Graphify refreshed the worktree graph to 9,673 nodes and 16,229 edges. A local path query linked `launch_chat()` and `chat_snapshot()` through their shared `Result` type; source and integration tests remain the authoritative evidence. The generated 21 MB graph output was removed after analysis. CocoIndex is unavailable because no `cocoindex` executable is installed in this environment.
- `docs/user-manual.md`, `docs/user-manual.ko.md`: operator-facing limits and usage.

The full local suite passed 1,000 tests with 14 ignored across 334 suites before the final editor-focus guard and base-branch rebase. On rebased head `70302a1b`, focused chat/startup/layout tests (19) and Windows dispatch tests (11) passed; the full local run was stopped at the five-minute bound after 933 passed and 14 ignored across 293 suites, so it has no final pass/fail result. Exact-head CI run `40529042439` is pending. Packaged macOS/Windows UI interaction remains unverified; this feature does not increase #9's accepted packaged workflow count until the exact Windows package persistence check passes.
