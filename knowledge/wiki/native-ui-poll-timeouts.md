# Native UI background polling and timeout policy

## Decision

The Rust desktop UI dispatches requests through a single-slot `RequestWorker` channel to `Room::dispatch` and receives typed results on a reply channel. This path has no page-side request deadline. Time spent while the native window is hidden or minimized therefore cannot expire a UI request.

Only existing scheduled Agent Room snapshots use a distinct background pending state. Their errors do not enter the user-action toast channel. Initial Workspace/Agent Room snapshots, explicit Workspace refreshes, approvals, and mutations retain visible errors. Task-detail and coordinator reads automatically triggered by a scheduled Room snapshot inherit the quiet background policy; late task-detail responses that no longer match the selection or an open Room are discarded quietly. The Workspace remains event-driven when idle; no idle poll loop was added.

Background refresh is gated by the live Room interaction state. A pending receipt in a snapshot does not by itself stop polling after the operator defers it; an open tool dialog or confirmation still does. This avoids freezing out a replacement receipt while keeping an active decision stable. The current poll dispatcher routes all settings-owned pending states through `Pending::handled_by_settings`, including chat reset and optimizer operations added after the original poll extraction.

The Agent Room repaint scheduler stores a monotonic `Instant`. If the OS delays repaint while the window is minimized, the first resumed frame sends at most one due poll and sets the next deadline to the current time plus the interval. It does not replay state-changing requests.

An operator-triggered tool-request refresh has a separate bounded follow-up. The UI captures the receipt's task and requests quiet snapshots once per second for up to 20 reads. If the coordinator has started the refresh but has not yet published the replacement receipt, an intermediate snapshot can temporarily omit the old receipt; while the run remains active, that gap does not cancel the follow-up. A pending replacement for the same task ends it, as does the run becoming inactive or the read bound being exhausted. Pending requests for unrelated tasks do not cancel it. The follow-up is created only by the operator's refresh action; ordinary scheduled reads remain paused while a pending receipt is actionable. Approve, reject, and refresh requests retain their existing revision and receipt binding, and no state-changing request is retried.

## JavaScript distinction

The JavaScript UI owns a client timeout and can report a `client_timeout` event with page visibility and measured wait time. The native Rust path does not create that event, so client-timeout diagnostics are not ported. See issue #221 and the JavaScript follow-up PR #225 for the page-timer investigation.

## Trace and verification

Trace: R-186 → BG-141 → SF-174 → UC-177 → FR-178/NFR-76 → QAS-184 → DDR-40 → DD-198.

Rust regression coverage checks visible initial snapshot failures, scheduled Agent Room polling suppression while a decision is open, resumed polling after a receipt is deferred, preserved explicit approve/refresh dispatch, bounded follow-up across the coordinator's receipt-replacement gap, same-task replacement detection, unrelated-task isolation, quiet stale task-detail replies, visible explicit refresh/action failures, and deadline rebasing after a late repaint. Exact PR run `40852587950` found that an older test still expected any snapshot-pending receipt to block refresh, despite the deferred-receipt behavior. The test now distinguishes an open dialog from a deferred receipt. It failed locally before the correction; after correction, focused targets pass 21 tests across source layout, native room poll, native probe, and tool dialog suites. Warning-denied all-target Clippy, rustfmt, diff checks, and the 300-line limit pass.

Exact-head workflow `40856147687` then passed both platform validation jobs and both native package jobs on pre-sync commit `03ef31a3`, including Windows packaged default-desktop UI automation. PR #228 still conflicted with the latest #9 base, so the existing issue branch was synchronized with #9 head `68238d04`; only the Wiki index and log required conflict resolution, and both branches' entries were retained. On the merged tree, nine focused targets pass 43 tests; warning-denied all-target Clippy, rustfmt, diff checks, and the 300-line limit pass. This latest merge head still needs exact macOS/Windows package verification before PR #228 or issue #221 can close.
