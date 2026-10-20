# Native review after a plan starts

The Workspace Review step must distinguish a proposal awaiting human approval from a phase that already started. Returning from a completed run can leave the stored proposal phase as `starting`; showing its task cards as `Pending` suggests that approval or execution is still available and misrepresents the run.

The native Review pane now checks the persisted proposal phase. For `starting`, it shows a localized explanation directing the user to Agent Room for results or to edit the goal before preparing another phase. It omits stale task cards and the old proposal digest. Other proposal phases retain the existing review rendering.

Regression evidence: `tests/native_ui_probe/review_width.rs::an_already_started_plan_is_not_presented_as_pending_after_returning_to_workspace` renders a completed run with `proposal.phase = starting` and asserts the explanation appears while stale `task-0` is absent. The notice now has a localized **Open Agent Room** button. It checks the workspace snapshot's `run_available`, sends the existing `open_run` request, and switches the same native window's request worker back to Agent Room. It does not spawn another process or window.

`tests/native_ui_start_journey.rs::review_approval_shows_start_feedback_then_opens_the_started_run` now exercises start, task completion, candidate approval and merge, return to Workspace, then reopening Agent Room. The locked all-target suite passed (775 passed, 14 ignored, 257 suites); all-target Clippy, rustfmt, diff checks, and the 300-line guard pass. Exact packaged macOS review and Windows behavior remain pending. This is a return-navigation subpath, not a new distinct end-to-end acceptance; the conservative packaged workflow count remains 1/25.
