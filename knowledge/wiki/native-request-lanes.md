# Native Request Lanes

## Purpose

Keep slow local reads from preventing user actions or state changes in the Rust native UI. Preserve the event-driven peer refresh path and keep host resource use predictable.

## Execution model

- A fixed pool of two workers handles known read-only requests.
- One worker handles state-changing requests and picker requests, preserving mutation order.
- A dedicated bounded worker handles synchronous network-bound work such as messenger connection checks, so it cannot occupy the state-change lane.
- Each foreground lane has a bounded queue of 16 requests. A full queue returns a short busy error instead of creating more threads.
- Periodic snapshots retain their separate single-flight worker and reply channel.
- Foreground replies carry monotonically increasing IDs. The UI discards replies older than the most recently accepted request.
- Successful state changes publish content-free invalidation events. Other native views coalesce notifications and request one authoritative snapshot; a view does not refresh itself for its own event.
- Unknown valid request kinds use the state-change lane. This conservative default avoids concurrent state mutation and ensures future mutations reach peer listeners.

## UI contract

The UI submits requests without waiting for execution. Worker completion requests a repaint; event listeners wake peer views; due event and recovery refreshes run even while unrelated user work is pending, except while a decision modal freezes its view. Snapshot application preserves pending operations, unsaved input, and errors.

## Regression coverage

`tests/native_ui_probe/worker_concurrency.rs` uses barriers to prove that two reads can execute concurrently, a state-changing action completes while a snapshot or messenger connection check is blocked, and state changes remain serialized. `tests/native_ui_probe/events.rs` verifies coalesced event fanout, source suppression, and read-only filtering.

## Constraints

The implementation uses Rust standard-library threads and channels. It adds no async runtime, watcher service, or unbounded task spawning. The fixed worker pool adds two read workers, one mutation worker, and one blocking-I/O worker per native view, in addition to the existing single-flight refresh worker.

## Event and recovery refresh during foreground work

A pending foreground response no longer prevents a peer-view invalidation or due recovery check from dispatching to the bounded snapshot worker. Active human decision modals still pause refresh for their view. The snapshot reply lane remains separate from foreground replies, and application preserves the pending action, dirty form data, and current error. Regressions: `tests/native_ui_probe/poll_events.rs` and `tests/native_ui_probe/renderer.rs`.

Workspace snapshot application compares each local setting draft with the previous saved snapshot. If the draft differs, that field remains local and is not replaced by the incoming event snapshot; fields still matching the previous saved value are refreshed from the new snapshot. The goal optimizer proposal is also retained while the goal draft is dirty. `tests/native_ui_probe/poll_background_pending.rs` verifies unsaved name and goal preservation while a clean repository field accepts the new saved value.

## Task-progress spool observation

The task-progress spool observer starts at 100 ms, doubles its delay up to 2 seconds while no bytes arrive, and resets to 100 ms as soon as output resumes. At process completion it continues draining while bytes remain, even when a final JSONL record is partial, then flushes the decoder. This lowers idle file/database reads without adding a runtime or dropping final progress.

## Coordinator process completion

The native request worker must never wait for a coordinator child or for terminal SQLite audit persistence. A named standard-library monitor thread owns `Child::wait`, updates the in-memory job state briefly, then writes the owner record and terminal action event outside the job-state lock. Integration tests wait for this monitor-owned terminal receipt and exit code; a second `waitpid` races the monitor and is not a valid completion check. Poll requests read the in-memory state and continue reporting `running` until terminal persistence succeeds; a persistence failure is surfaced as a failed job. If monitor thread creation fails, the existing `try_wait` fallback remains available. Regression: `tests/support/controller_job_monitor.rs` holds a SQLite write lock after the child exits and verifies that `Controller::poll` remains responsive until the audit can be saved.

## Cross-process invalidation

The Rust EventBus handles same-process view updates. A spawned AgentConductor command receives an ephemeral loopback UDP address, a per-bridge token, and its target surface through `AC_UI_EVENT_*` variables; progress and command completion send a small authenticated invalidation packet. Provider, MCP, quota, and Git children have these variables removed before spawn. The receiver is bound only to `127.0.0.1`, runs on a named thread, and sleeps in `recv_from` with a 30-second shutdown timeout. When the bridge is unavailable or external code cannot signal, the existing file observer is the recovery path; it checks metadata on a fixed 30-second interval and publishes an invalidation when it detects a change. It does not reset to a shorter cadence or assemble snapshots. It fingerprints the primary report/startup file plus sibling `state.sqlite` and `state.sqlite-wal`. Regressions cover valid-token delivery, wrong-token rejection, target-surface fanout, environment scoping, the fixed recovery interval, and SQLite-only changes.

## Closed PR #255 regression audit — 2026-11-03

PR #255's exact source paths were partly superseded by module consolidation. The behavior is covered in the current tree: `tests/native_ui_probe/events.rs` checks surface-scoped/coalesced invalidations and listener lifetimes; `worker_concurrency.rs` checks parallel bounded reads, independent foreground actions, isolated background snapshots, and serialized mutations; `poll_events.rs` and `poll_background_pending.rs` cover event/recovery refresh while preserving pending work and unsaved edits; `startup_chat.rs` covers persisted chat history and the timeout/revision regression; `windows_workspace_reopen.rs` covers saved-workspace reopening.

Local verification on 2026-11-03: `cargo test --locked --offline --features native-ui-probe --test native_ui_probe --test ui --test startup_chat --test windows_coordinator_chat --test launch_workspace --test native_ui_static_labels` passed 27 tests across 6 test targets. This is source-level regression evidence only. Historical exact-package evidence attached to the closed PR is not present in the current candidate, so the result does not satisfy packaged Windows/macOS acceptance. Issue #9 remains 22/50 overall, with packaged workflows 22/25 and shipped/default native UI parity 0/25.
