# Event-driven native updates

## Current behavior

The native UI uses bounded typed invalidations to wake only subscribed surfaces. These events enqueue targeted reads on a background request lane; the render callback only drains completion/event channels and schedules repaint. There is no repeating whole-screen snapshot timer. Explicit retry deadlines, configured quota refresh, and tool follow-up remain one-shot or separately scheduled work.

Local UI child processes are supervised by one named monitor thread per active child. It blocks in `Child::wait()` off the UI/request workers, persists terminal state without holding the controller state lock, then publishes one completion invalidation. State-changing commands remain serialized at the state owner; independent reads use bounded worker lanes and stale replies are rejected by request/revision identity.

Earlier iterations used periodic full snapshots as recovery. That fallback was removed from the UI path in the follow-up recorded below; the separate authenticated observer remains responsible for external process/filesystem changes with bounded backoff.

## Requirement and decision

GitHub #257 records R-192: local Rust state changes should publish typed invalidation events; subscribed views should refresh only relevant data and accept results only when their revision is current. Delivery must be bounded and non-blocking, duplicate invalidations coalesced, conflicting mutations serialized at the state owner, and disconnected subscribers ignored. Long-running work reports completion/progress independently. External CLI or filesystem changes may use one independent bounded recovery observer with backoff.

Trace: R-192 → BG-147 → SF-180 → UC-183 → FR-184/NFR-79 → QAS-190 → DDR-44 → DD-204.

## Implementation status

- Plan and architecture records are updated.
- A bounded `std::sync::mpsc` event hub filters subscribers by Workspace or Agent Room and coalesces a full one-slot subscriber queue. Publishers atomically set surface bits and `try_send` one wake token to a capacity-one channel, then return. One lightweight fan-out thread drains and coalesces those bits, touches the subscriber registry, and calls `egui::Context::request_repaint` for matching surfaces. Registry contention delays delivery on the fan-out thread rather than blocking the state owner or losing the event.
- Successful native UI dispatches publish an invalidation; the matching view listens on its bounded receiver and asks the existing independent read lane for one background result.
- Each foreground request receives a monotonic ID. The reply receiver discards completed responses whose IDs are no longer current, preventing the two request lanes from applying an earlier snapshot after a later action.
- Each background snapshot records the foreground request ID that was current when it began. If a newer foreground request starts before the snapshot returns, the UI drops that response instead of overwriting newer form state.
- There is no recurring whole-screen recovery deadline in the UI. Event subscribers trigger refresh immediately. Timed recovery is limited to explicit one-shot retry work, quota refresh, tool follow-up, and the independent bounded external-change observer.
- The dispatcher has bounded lanes for reads and mutations, plus a separate single-flight background snapshot lane used only when requested by an event or explicit deadline. Reads have two workers so one slow read cannot stall another; mutations remain serialized by the shared state owner. Each foreground lane queues up to 16 requests with `try_send`, so the render callback never waits on dispatch. TDD covers snapshot isolation, worker concurrency, and queuing.
- A UI child-job terminal transition is observed by a dedicated thread blocked in `Child::wait()`, not by UI polling. It updates in-memory terminal state, releases the controller lock before file/SQLite persistence, then publishes one invalidation.
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

## No-periodic-snapshot and child-wait follow-up — 2026-10-30

Removed the repeating 30-second Workspace/Agent Room snapshot cadence and its `room::State.next_refresh` timer. Event callbacks now route coordinator chat, optimizer, metadata, MCP, start/stop and generic invalidations to their relevant background result request. An invalidation that races a snapshot supersedes it and queues targeted work. CLI child exit now comes from `Child::wait()` on the monitor thread; UI reads no longer call `try_wait()`.

TDD regressions first failed because idle render scheduled a snapshot, coordinator chat invalidation requested a generic snapshot, and controller completion depended on calling `poll()`. Their green counterparts are in `tests/native_ui_probe/renderer.rs`, `tests/native_ui_probe/app.rs`, and `tests/support/controller_events.rs`. Local full-suite status is recorded in `knowledge/wiki/log.md` after completion.

## Exact package checkpoint — 2026-10-30

The pushed source `62efb31c8725d93fd1bb10021cad170cf3a77533` builds and packages for `aarch64-apple-darwin`. Manifest/source matching, embedded binary digest, archive integrity, packaged `--help`, and package smoke passed. Evidence: `docs/evidence/issue257-macos-package-62efb31-20261030.json`.

The packaged app was launched once but opened the user's saved workspace. No workspace or agent action was taken, and its visible controls appeared disabled; this observation is unresolved and earns no GUI workflow credit. Exact packaged UI journeys and Windows packaging remain open acceptance work for #257 and #9.

The follow-up fix at `e7f712a60c83727ddeb27c09da0895624a68bf14` was packaged and launched after closing the earlier package process. In that exact build, stale quota observations were present and workspace fields, agent edit, quota refresh, and save controls were enabled. No saved data was changed. This verifies the quota-refresh lock regression at launch, not the full native UI workflow; Windows remains pending. Evidence: `docs/evidence/issue257-macos-package-e7f712a-20261030.json`.

The request-queue fix at `cd2b1d4850f9d77c1814e1e0b95ab2549f62bd31` was packaged, manifest-validated, and smoke-tested across shared/worktree isolation with Git and folder projects. The exact package was launched after closing the prior AgentConductor process; one v0.1.37 app window opened the saved workspace with controls enabled and no pending edits. No workspace data was changed. This confirms package identity and launch only; full UI interaction and Windows acceptance remain pending. Evidence: `docs/evidence/issue257-macos-package-cd2b1d48-20261030.json`.

The event-fanout implementation at `01bc3c55338ae5c02e9a0cc46761bd16f383b6b8` packaged for `aarch64-apple-darwin`; manifest/source matching, packaged `--help`, and all four shared/worktree × Git/folder mock-agent smoke cases passed. The package process was started with a disposable workspace, but the UI accessibility bridge timed out, so no interactive UI behavior is claimed. The Windows VM remained on its existing update screen and was not changed. Evidence: `docs/evidence/issue257-macos-package-01bc3c55-20261030.json`.

This event-fanout slice adds a second bounded read worker and moves event delivery to a dedicated asynchronous listener. Its regression suite passes 83/83 unit tests; package-level interactive UI and Windows acceptance remain open.

## Exact isolated package startup — 2026-10-30

The package from `189f6fadeecda8f4b7c92eb269dfb0d455afb9c5` was opened by its exact app path with `ui-new` targeting a disposable `/private/tmp` workspace. The native window showed an empty workspace and zero agents; the temporary startup configuration was empty. The registered older app was closed, leaving one package process. No workspace data was changed. This proves package identity and isolated startup only; it earns no packaged workflow acceptance. Exact event-driven UI interaction and Windows package acceptance remain open. Evidence: `docs/evidence/issue257-macos-package-189f6fad-20261030.json`.

## Exact packaged workflow interaction — 2026-10-30

The same `189f6fadeecda8f4b7c92eb269dfb0d455afb9c5` package ran a disposable workspace with four deterministic Rust mock profiles. After correcting fixture-only prompt and role assumptions, the plan reached review, three scoped tasks completed, all three `git diff --check` commands passed, the app exposed the diff and test results, and the operator approval and merge gates completed. The UI showed the run as completed.

A three-second custom quota subprocess also completed with exit code 0 and wrote its normalized 75%-remaining snapshot to the isolated quota cache. While it was running, switching the workspace step completed in 0.41 seconds, so the slow read did not block that UI action. The exact package was later bound by its bundle path and its accessibility tree confirmed the disposable workspace and mock profiles. The tested Agent Roles sidebar intentionally renders quota badges only for Claude and Codex, so that custom profile cannot verify visible quota-event refresh. Repeat on a supported badge-rendering surface or with a deterministic first-party-profile fixture. Do not close #257 or award issue #9 packaged UI credit from this evidence. Windows package validation remains open. Evidence: `docs/evidence/issue257-packaged-event-ui-20261030.json`.

## Event-to-metadata regression — 2026-10-30

Added `tests/native_ui_probe/poll_events.rs::metadata_event_refreshes_quota_snapshot_asynchronously`. It publishes an invalidation through the subscribed workspace event hub while the current snapshot reports a running metadata operation, verifies that the Probe retrieves a new snapshot through its asynchronous poll worker, and renders the resulting first-party quota badge in the native sidebar accessibility tree. Focused test: 1/1 passed; native UI library filter: 59/59; `source_layout`: 2/2; Clippy with `-D warnings`, rustfmt and `git diff --check` passed. The source poll module is 51 lines and the regression test is 69 lines. An earlier all-target filtered invocation was interrupted after its native library tests while traversing unrelated test targets (exit 130); that invocation is not claimed as a full-suite pass. This strengthens source-to-view event evidence; exact packaged UI and Windows checks remain open.

## Links

- Requirements: `docs/01-requirements.md` through `docs/06-design-decisions.md`, R-192 / DD-204.
- Checklist: `docs/07-implementation-plan.md`, “Event-driven native updates — GitHub #257 / issue #9”.
- Previous polling isolation: [Native UI event and polling lanes](native-event-polling.md).
