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
- Each foreground request receives a monotonic ID. The reply receiver discards completed responses whose IDs are no longer current, preventing the two request lanes from applying an earlier snapshot after a later action.
- Each background snapshot records the foreground request ID that was current when it began. If a newer foreground request starts before the snapshot returns, the UI drops that response instead of overwriting newer form state.
- Periodic whole-screen refresh is now only a 30-second recovery deadline regardless of running/idle status. Event subscribers repaint and trigger snapshots immediately, so active workflows no longer cause 1-second polling and idle work no longer causes 5-second polling; quota refresh remains on its own configured schedule.
- The foreground dispatcher has two independent bounded lanes: a single-flight poll lane and two worker lanes for UI commands. Each command lane queues up to 16 requests with `try_send`, so input never waits on a worker and short bursts do not fail just because one request is still running. TDD covers both poll isolation and queued commands.
- A UI child-job terminal transition publishes an event when the controller observes process exit. The event wakes the view to read the committed state.
- Every supervised CLI child notifies its parent UI once at command exit through the same authenticated loopback bridge. The send is non-blocking and is a no-op when the child was not launched by a UI supervisor.
- When an invalidation arrives during a background snapshot, the view drops that in-flight result and queues one fresh snapshot. This prevents a pre-mutation read from briefly overwriting newer view state.
- The internal workflow child inherits an ephemeral loopback UDP address and random token. Its supervised stdout-spool reader sends non-blocking invalidations after recording agent progress; the parent wakes only the surface that launched the work. Provider, MCP, quota, and helper subprocesses have these credentials stripped.
- If the spool observer sees no output, its polling delay increases from 100 ms to at most 2 s; output resets it to 100 ms. The UI itself is woken by events and uses a separate snapshot worker.
- Advisory quota metadata lookup and its read-only snapshot refresh do not lock workspace inputs or navigation. A newer foreground action supersedes their replies; state-changing operations and active workflow guards still serialize as before.
- The same listener runs one bounded recovery scan for external changes. It compares `report.json`, the SQLite state/WAL, and files declared in active tasks' worktree scopes (at most 512 paths and depth 8). It emits only the owning surface's invalidation; idle scans back off from 1 s to 30 s and reset after a detected change. If loopback socket setup fails, one dedicated recovery worker maintains this behavior. The native render callback does no filesystem work.
- Local CocoIndex CLI and Python module are unavailable in the current environment (`python3 -c 'import cocoindex'` fails with `ModuleNotFoundError`). Graphify was refreshed and used to trace the worker/snapshot path. Do not claim CocoIndex execution.
- Enhancement #257 is open. The package acceptance boundary remains macOS and Windows exact-build UI journeys.

## Verification design

Use deterministic channel and loopback tests for authenticated event delivery, child-exit notification, bounded non-blocking sends, subscriber disconnect, coalescing, selective surface delivery, superseded request and snapshot rejection, external declared-scope file changes, and independent foreground progress while another request is blocked. Verify the recovery path's scan bounds and idle backoff. Avoid a thread per notification and avoid work in egui's render callback.

## Exact package checkpoint — 2026-10-30

The pushed source `62efb31c8725d93fd1bb10021cad170cf3a77533` builds and packages for `aarch64-apple-darwin`. Manifest/source matching, embedded binary digest, archive integrity, packaged `--help`, and package smoke passed. Evidence: `docs/evidence/issue257-macos-package-62efb31-20261030.json`.

The packaged app was launched once but opened the user's saved workspace. No workspace or agent action was taken, and its visible controls appeared disabled; this observation is unresolved and earns no GUI workflow credit. Exact packaged UI journeys and Windows packaging remain open acceptance work for #257 and #9.

The follow-up fix at `e7f712a60c83727ddeb27c09da0895624a68bf14` was packaged and launched after closing the earlier package process. In that exact build, stale quota observations were present and workspace fields, agent edit, quota refresh, and save controls were enabled. No saved data was changed. This verifies the quota-refresh lock regression at launch, not the full native UI workflow; Windows remains pending. Evidence: `docs/evidence/issue257-macos-package-e7f712a-20261030.json`.

The request-queue fix at `cd2b1d4850f9d77c1814e1e0b95ab2549f62bd31` was packaged, manifest-validated, and smoke-tested across shared/worktree isolation with Git and folder projects. The exact package was launched after closing the prior AgentConductor process; one v0.1.37 app window opened the saved workspace with controls enabled and no pending edits. No workspace data was changed. This confirms package identity and launch only; full UI interaction and Windows acceptance remain pending. Evidence: `docs/evidence/issue257-macos-package-cd2b1d48-20261030.json`.

## Links

- Requirements: `docs/01-requirements.md` through `docs/06-design-decisions.md`, R-192 / DD-204.
- Checklist: `docs/07-implementation-plan.md`, “Event-driven native updates — GitHub #257 / issue #9”.
- Previous polling isolation: [Native UI event and polling lanes](native-event-polling.md).
