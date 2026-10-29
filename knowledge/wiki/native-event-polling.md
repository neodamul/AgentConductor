# Native UI event and polling lanes

## Decision

Keep state-changing commands serialized on the foreground request worker. Send recurring read-only snapshots through a dedicated bounded polling worker with its own completion channel. The UI listens for either completion channel and calls `request_repaint()` from the worker only when a reply is ready; it does not run a continuous poll loop. A single `poll_pending` value coalesces timer ticks while a poll is queued, running, or awaiting UI application. A poll reply waits in its channel while a foreground operation is pending, so it cannot overwrite newer UI state.

Recurring workspace, Agent Room, coordinator-chat, optimizer, metadata, and MCP status snapshots use the poll lane. User-triggered mutations and explicit commands remain on the serialized lane. This keeps ordering and shared-state safety for mutations while preventing a slow recurring read from occupying the foreground request worker.

`Startup::dispatch` still performs its short metadata tick and settle transition under the saved-settings mutex. For a snapshot request it then clones `Saved` and releases the mutex before reading plan files, reports, chat artifacts, task activity, icons, or MCP details. A concurrent mutation may make that snapshot stale; its revision remains attached and the existing revision checks reject stale mutations.

No async runtime dependency, always-on loop, new server, or second state store is introduced. The two worker threads share the existing Rust dispatch path and use bounded request channels plus completion events.

## Trace and verification

Trace: issue #189; DDR-43; DD-203. Implementation branch: `codex/issue-189-chat-package-current`.

Local focused evidence: `foreground_dispatch_completes_while_a_periodic_poll_is_blocked` 1/1; `poll_reply_stays_pending_until_foreground_operation_finishes` 1/1; `repeated_poll_ticks_coalesce_while_one_poll_is_pending` 1/1; existing native poll policy 6/6; `snapshot_clone_releases_settings_lock_before_rendering` 1/1; `startup_chat` 5/5; `metadata_failure` 1/1; `source_layout` 2/2; warning-denied library Clippy and rustfmt pass. Full suite and exact packaged UI acceptance are not claimed. Windows package verification remains outstanding.
