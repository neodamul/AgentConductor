# Event-driven native updates

## Current behavior

Before this implementation slice, the native UI used one serialized foreground request worker and an independent single-flight poll worker. The poll thread protected controls from a stalled scheduled snapshot, but Workspace and Agent Room still relied on periodic snapshot reads. Asynchronous placement alone did not provide event-driven state propagation or prevent a slow foreground dispatch from delaying a later request.

## Requirement and decision

GitHub #257 records R-192: local Rust state changes should publish typed invalidation events; subscribed views should refresh only relevant data and accept results only when their revision is current. Delivery must be bounded and non-blocking, duplicate invalidations coalesced, conflicting mutations serialized at the state owner, and disconnected subscribers ignored. Long-running work reports completion/progress independently. External CLI or filesystem changes may use one independent bounded recovery observer with backoff.

Trace: R-192 → BG-147 → SF-180 → UC-183 → FR-184/NFR-79 → QAS-190 → DDR-44 → DD-204.

## Implementation status

- Plan and architecture records are updated.
- A bounded `std::sync::mpsc` event hub now filters subscribers by Workspace or Agent Room, coalesces a full one-slot queue, and calls `egui::Context::request_repaint` without adding an event thread. Publisher delivery uses `try_lock` plus `try_send`; transient registry contention cannot block a state owner, and later snapshots/recovery reconcile invalidations.
- Successful native UI dispatches publish an invalidation; the matching view listens on its bounded receiver and asks the existing independent poll worker for one background snapshot.
- Periodic whole-screen refresh is now only a 30-second recovery deadline regardless of running/idle status. Event subscribers repaint and trigger snapshots immediately, so active workflows no longer cause 1-second polling and idle work no longer causes 5-second polling; quota refresh remains on its own configured schedule.
- The serialized foreground lane is now two bounded worker lanes. TDD proved a slow request on one lane does not delay a quick request on the other.
- A UI child-job terminal transition publishes an event when the controller observes process exit. The event wakes the view to read the committed state.
- When an invalidation arrives during a background snapshot, the view drops that in-flight result and queues one fresh snapshot. This prevents a pre-mutation read from briefly overwriting newer view state.
- The internal workflow child inherits an ephemeral loopback UDP address and random token. Its supervised stdout-spool reader sends non-blocking invalidations after recording agent progress; the parent wakes only the surface that launched the work. Provider, MCP, quota, and helper subprocesses have these credentials stripped.
- If the spool observer sees no output, its polling delay increases from 100 ms to at most 2 s; output resets it to 100 ms. The UI itself is woken by events and uses a separate snapshot worker.
- The same listener runs one bounded recovery scan for external changes. It compares `report.json`, the SQLite state/WAL, and files declared in active tasks' worktree scopes (at most 512 paths and depth 8). It emits only the owning surface's invalidation; idle scans back off from 1 s to 30 s and reset after a detected change. If loopback socket setup fails, one dedicated recovery worker maintains this behavior. The native render callback does no filesystem work.
- Local CocoIndex CLI and Python module are unavailable in the current environment (`python3 -c 'import cocoindex'` fails with `ModuleNotFoundError`). Graphify was refreshed and used to trace the worker/snapshot path. Do not claim CocoIndex execution.
- Enhancement #257 is open. The package acceptance boundary remains macOS and Windows exact-build UI journeys.

## Verification design

Use deterministic channel and loopback tests for authenticated event delivery, bounded non-blocking sends, subscriber disconnect, coalescing, selective surface delivery, stale snapshot rejection, external declared-scope file changes, and independent foreground progress while another request is blocked. Verify the recovery path's scan bounds and idle backoff. Avoid a thread per notification and avoid work in egui's render callback.

## Links

- Requirements: `docs/01-requirements.md` through `docs/06-design-decisions.md`, R-192 / DD-204.
- Checklist: `docs/07-implementation-plan.md`, “Event-driven native updates — GitHub #257 / issue #9”.
- Previous polling isolation: [Native UI event and polling lanes](native-event-polling.md).
