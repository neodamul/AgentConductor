# Review evidence while a coordinator run is active

## User-visible contract

After a candidate is approved, its coordinator job can still report `running` while it completes its transition. During that interval the user must still be able to open `Review changes` and inspect the diff and test evidence. A pending request still disables the control. Approval, merge, and workspace actions remain subject to the existing running-state guard.

## Cause and implementation

The Agent Room next-step renderer applied one blanket `job.state == running` gate to every action. The UI Automation trace showed `Review changes` disabled even though the matching evidence was read-only and the candidate review panel remained available. The renderer now allows only `NextAction::Review` through the running-state gate; it does not relax the pending-request guard or state-changing actions.

## Regression evidence

`tests/native_room_next_step.rs` creates an approved candidate with `job.state` still `running`, no current evidence, and asserts that `Review changes` is enabled and dispatches the evidence event. The test failed before the renderer change and passes after it. The focused suite passes 6/6; focused Clippy, rustfmt, diff hygiene, and the 300-line source/test limit pass. Exact Windows package execution is still required to confirm the UI Automation journey.

## Provenance

- Discovered by exact Windows package run `41369091868` on root head `524e661b`.
- Rust implementation is tracked in issue #9 and the existing root PR #22.
