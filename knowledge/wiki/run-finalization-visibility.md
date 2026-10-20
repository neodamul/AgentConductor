# Run finalization visibility

Keep candidate finalization on the primary run board as three distinct stages: Verify, Merge, and Workspace. Approval is a review decision; it does not itself integrate or apply the candidate. A running merge/apply job may coexist with a report state of `approved`, so derive the busy presentation from the existing job snapshot (`kind=merge`, `state=running`) rather than starting another poller.

The main run status and action area must remain visibly active during merge. Disable repeated merge requests while the job is running, keep the approved candidate and reviewed evidence bound to the same revision, and show Workspace only after the report reaches `completed` or `merged`. Keep stage/status strings in both English and Korean catalogs.

Trace: R-156 → BG-111 → SF-143 → UC-146 → FR-141 → QAS-152 → DD-166. GitHub #50.

Rust port validation: see the #50 checklist in `docs/07-implementation-plan.md`; do not close the issue before the issue-linked Rust PR has passed review, checks, and packaged visual QA.
