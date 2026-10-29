# Event-driven native updates

## Current behavior

Before this implementation slice, the native UI used one serialized foreground request worker and an independent single-flight poll worker. The poll thread protected controls from a stalled scheduled snapshot, but Workspace and Agent Room still relied on periodic snapshot reads. Asynchronous placement alone did not provide event-driven state propagation or prevent a slow foreground dispatch from delaying a later request.

## Requirement and decision

GitHub #257 records R-192: local Rust state changes should publish typed invalidation events; subscribed views should refresh only relevant data and accept results only when their revision is current. Delivery must be bounded and non-blocking, duplicate invalidations coalesced, conflicting mutations serialized at the state owner, and disconnected subscribers ignored. Long-running work reports completion/progress independently. External CLI or filesystem changes may use one independent bounded recovery observer with backoff.

Trace: R-192 → BG-147 → SF-180 → UC-183 → FR-184/NFR-79 → QAS-190 → DDR-44 → DD-204.

## Implementation status

- Plan and architecture records are updated.
- A bounded `std::sync::mpsc` event hub now filters subscribers by Workspace or Agent Room, coalesces a full one-slot queue, and calls `egui::Context::request_repaint` without adding an event thread.
- Successful native UI dispatches publish an invalidation; the matching view listens on its bounded receiver and asks the existing independent poll worker for one background snapshot.
- The serialized foreground lane is now two bounded worker lanes. TDD proved a slow request on one lane does not delay a quick request on the other.
- A UI child-job terminal transition publishes an event when the controller observes process exit. The event wakes the view to read the committed state.
- Workflow/agent progress written by an external child process still depends on the existing periodic recovery snapshots. The independent backoff observer and producer-side progress events are not implemented; this slice is not a complete event-driven replacement for polling.
- Local CocoIndex CLI and Python module are unavailable in the current environment (`python3 -c 'import cocoindex'` fails with `ModuleNotFoundError`). Graphify was refreshed and used to trace the worker/snapshot path. Do not claim CocoIndex execution.
- Enhancement #257 is open. The package acceptance boundary remains macOS and Windows exact-build UI journeys.

## Verification design

Use deterministic std channel tests for event fan-out, bounded non-blocking sends, subscriber disconnect, event coalescing, selective surface delivery, and independent foreground progress while another request is blocked. Test revision rejection at the view boundary. Keep any eventless external observer on a separate bounded worker; verify that fallback cadence backs off while idle. Avoid a thread per notification and avoid work in egui's render callback.

## Links

- Requirements: `docs/01-requirements.md` through `docs/06-design-decisions.md`, R-192 / DD-204.
- Checklist: `docs/07-implementation-plan.md`, “Event-driven native updates — GitHub #257 / issue #9”.
- Previous polling isolation: [Native UI event and polling lanes](native-event-polling.md).
