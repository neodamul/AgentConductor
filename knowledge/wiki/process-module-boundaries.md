# Process module boundaries

## Scope

Source: #472, R-35 / DD-41. The public `process` facade remains stable so planner, workflow, quota and recovery callers retain their existing imports.

- `src/process.rs`: public types and entry points.
- `src/process/ownership.rs`: durable ownership receipts and proof of quiescence.
- `src/process/supervision.rs`: child execution, unpaused deadlines, cancellation, stream draining and signal escalation.
- Existing control, containment and Windows Job modules retain their responsibilities.

## Preserved invariants

A stopped, timed-out or failed-control attempt cancels callbacks before escalation; descendants do not outlive the attempt. Unknown ownership never counts as quiescent. Ownership writes retain create-new, sync and rename behavior. Windows process absence remains delegated to the existing Job helper. No approval or validation is removed by relocation.

## Verification

The area-specific layout test first failed at 467 lines, then passed with 119/82/276-line modules. Existing process, timeout, ownership and async-control tests cover behavior. Full-suite and platform receipts belong to the area PR; a local macOS pass alone does not prove Windows behavior.
