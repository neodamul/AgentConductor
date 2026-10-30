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
