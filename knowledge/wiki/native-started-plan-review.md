# Native review after a plan starts

The Workspace Review step must distinguish a proposal awaiting human approval from a phase that already started. Returning from a completed run can leave the stored proposal phase as `starting`; showing its task cards as `Pending` suggests that approval or execution is still available and misrepresents the run.

The native Review pane now checks the persisted proposal phase. For `starting`, it shows a localized explanation directing the user to Agent Room for results or to edit the goal before preparing another phase. It omits stale task cards and the old proposal digest. Other proposal phases retain the existing review rendering.

Regression evidence: `tests/native_ui_probe/review_width.rs::an_already_started_plan_is_not_presented_as_pending_after_returning_to_workspace` renders a completed run with `proposal.phase = starting` and asserts the explanation appears while the stale `task-0` row is absent. The exact v0.1.31 package reproduced the stale Pending rows on 2026-10-20. At this checkpoint the patched package, Windows behavior, and exact-head remote CI remain unverified. This is not a new end-to-end workflow acceptance; the conservative packaged workflow count remains 1/25.
