# Native UI background polling and timeout policy

## Decision

The Rust desktop UI dispatches requests through a single-slot `RequestWorker` channel to `Room::dispatch` and receives typed results on a reply channel. This path has no page-side request deadline. Time spent while the native window is hidden or minimized therefore cannot expire a UI request.

Only existing scheduled Agent Room snapshots use a distinct background pending state. Their errors do not enter the user-action toast channel. Initial Workspace/Agent Room snapshots, explicit Workspace refreshes, approvals, and mutations retain visible errors. Task-detail and coordinator reads automatically triggered by a scheduled Room snapshot inherit the quiet background policy; late task-detail responses that no longer match the selection or an open Room are discarded quietly. The Workspace remains event-driven when idle; no idle poll loop was added.

The Agent Room repaint scheduler stores a monotonic `Instant`. If the OS delays repaint while the window is minimized, the first resumed frame sends at most one due poll and sets the next deadline to the current time plus the interval. It does not replay state-changing requests.

## JavaScript distinction

The JavaScript UI owns a client timeout and can report a `client_timeout` event with page visibility and measured wait time. The native Rust path does not create that event, so client-timeout diagnostics are not ported. See issue #221 and the JavaScript follow-up PR #225 for the page-timer investigation.

## Trace and verification

Trace: R-177 → BG-132 → SF-165 → UC-168 → FR-163/NFR-73 → QAS-174 → DDR-37 → DD-188.

Rust regression coverage checks visible initial snapshot failures, scheduled Agent Room polling, quiet stale task-detail replies, visible explicit refresh/action failures, and deadline rebasing after a late repaint. Changed Rust files stay at or below 300 lines. Exact macOS/Windows package verification is required before merging the Rust port into the migration branch.
