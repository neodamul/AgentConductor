# Coordinator-directed operator feedback

The Agent Room routes task feedback to the selected task and coordinator requests to the recorded coordinator profile. The coordinator panel names that profile and owns a separate draft. A coordinator request is limited to 8 KiB and is sent through the existing Rust action broker and configured CLI authentication.

The request lifecycle (`queued`, `processing`, `completed`, `failed`) is recorded in the run's SQLite audit stream. The coordinator receives the operator's request alongside frozen run evidence through the existing read-only review process. A proposal must pass the current task/evidence/charter validator. Only an evidence-backed `revise` proposal can rerun affected work; an informational request leaves task attempts unchanged. The coordinator cannot approve or merge.

The audit-derived coordinator view refreshes when its event count changes; this adds no resident service or polling process. Waiting for an active run's file lease is bounded by the configured workflow timeout (up to 30 minutes); expiry is retained as a failed request. If the room briefly reacquires its lease after the preflight wait, the review retries only that transient lease conflict within the same deadline. Other errors fail immediately.

## Trace and verification

- Requirement/design trace: R-173 → BG-128 → SF-161 → UC-164 → FR-159/NFR-69 → QAS-170 → DDR-33 → DD-184; GitHub #165, under issue #9.
- Implementation commit: `f7dccad`; AgentMemory record: `mem_mvjx7viw_19e9423e6053`.
- Evidence: `../raw/2026-10-23-issue165-coordinator-feedback.md`.
- Local verification before the lease-race follow-up: 920 tests passed, 14 ignored, zero failures across 310 suites in serial mode; warning-denied all-target Clippy, rustfmt, diff check, and the 300-line layout check passed.
- Lease-race regression follow-up: 3 retry unit tests, 3 coordinator workflow tests, 2 source-layout tests, and both CLI workflow tests passed alone; all-target warning-denied Clippy, rustfmt, and diff check passed. The final complete serial offline suite passed 923 tests, 14 ignored, 0 failed across 310 suites in 367.20 seconds. An earlier attempt hit one transient `DirectoryNotEmpty` cleanup failure in the CLI workflow test; the isolated suite and final complete run passed.
- Exact package `b298ae79774e` passed package validation and `--help`. Its native UI showed the recorded fixture coordinator, enforced empty/nonempty send-button state, and retained the candidate at human approval with 3/3 tasks and tests complete. Separately, the exact packaged CLI sent a request on a copied run to a deterministic read-only Rust reviewer; the SQLite audit recorded queued → processing → completed and an evidence-backed accept response. The copied candidate remained at human approval, with no approval or merge. Three fixture-only setup attempts failed before the successful dispatch. The successful lifecycle was not rendered in the UI, and Windows UI verification remains open. See `../evidence/issue9-b298ae7-coordinator-dispatch.json` for details.
- A render regression now verifies that the coordinator panel presents the latest request state for queued, processing, completed, and failed snapshots; it keeps only the newest status in the input panel while Activity displays history. The focused feedback/judgment render suites pass 10 tests. This does not close the separate packaged UI dispatch or Windows checks.
- Exact package UI dispatch was subsequently verified on v0.1.36 (`b298ae79774e`) using a copied run and a deterministic read-only local Rust reviewer. The operator sent a request in the Agent Room; the packaged UI showed queued, processing, completed, and the evidence-backed response. The run remained `waiting_approval` with 3/3 tasks/tests complete, and no approval or merge was created. The original run was unchanged; no provider CLI or external service ran. This adds distinct workflow 10/25. Windows UI verification remains open. Evidence: `../evidence/issue9-b298ae7-packaged-ui-coordinator-request.json`.
- The parallel full-suite attempt had one existing settings-dialog visibility test failure; the test passed alone and in the serial full suite. Exact packaged UI and Windows validation remain open issue #9 gates.

## Related assets

- [[native-inspector-and-judgments]]
- [[native-room-failure-feedback]]
- [[typed-coordinator-amendments]]
