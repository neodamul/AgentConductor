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

## Request-timeout handling (#234)

The native Rust request worker keeps the UI responsive for coordinator chat by taking a
read-only fast path. It checks the existing job and retained process ownership, then starts
the chat worker without running the Git `HEAD` lookup used by mutation revision guards. The
request revision is retained in the audit record, but chat does not mutate the reviewed
workspace and therefore does not reject a send because a repository lookup is slow.

`Room::dispatch` writes bounded timing evidence to
`~/.agentconductor/request-diagnostics.log`: every chat request has `start` and `done` lines;
other requests are recorded only when they take at least one second. A client timeout notice
can be recorded without changing the request result. The log contains request kind, byte size,
elapsed time, success, and a bounded error—not prompt contents—and rotates at 256 KiB.

The focused regression uses a Git wrapper that sleeps for 20 seconds on `rev-parse HEAD`; the
chat request must still be accepted in under two seconds. Exact packaged Windows/macOS
behavior and the user's failing `start kind=chat` / `done kind=chat ms=...` evidence remain
separate verification gates.

PR #239 was rebased onto current #9 head `28df83945269254fc669bea602bdccab99fe6ab6`.
The overlapping implementation-plan text was the only conflict; both issue checklists were
preserved. Focused startup-chat, diagnostics, and source-layout tests pass after rebase, as do
warning-denied all-target Clippy and formatting. Exact macOS/Windows package verification is
still required to confirm the packaged request path.
