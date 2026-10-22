# Coordinator-directed operator feedback

The Agent Room routes task feedback to the selected task and coordinator requests to the recorded coordinator profile. The coordinator panel names that profile and owns a separate draft. A coordinator request is limited to 8 KiB and is sent through the existing Rust action broker and configured CLI authentication.

The request lifecycle (`queued`, `processing`, `completed`, `failed`) is recorded in the run's SQLite audit stream. The coordinator receives the operator's request alongside frozen run evidence through the existing read-only review process. A proposal must pass the current task/evidence/charter validator. Only an evidence-backed `revise` proposal can rerun affected work; an informational request leaves task attempts unchanged. The coordinator cannot approve or merge.

The audit-derived coordinator view refreshes when its event count changes; this adds no resident service or polling process. Waiting for an active run's file lease is bounded by the configured workflow timeout (up to 30 minutes); expiry is retained as a failed request. If the room briefly reacquires its lease after the preflight wait, the review retries only that transient lease conflict within the same deadline. Other errors fail immediately.

## Trace and verification

- Requirement/design trace: R-173 → BG-128 → SF-161 → UC-164 → FR-159/NFR-69 → QAS-170 → DDR-33 → DD-184; GitHub #165, under issue #9.
- Implementation commit: `f7dccad`; AgentMemory record: `mem_mvjx7viw_19e9423e6053`.
- Evidence: `../raw/2026-10-23-issue165-coordinator-feedback.md`.
- Local verification before the lease-race follow-up: 920 tests passed, 14 ignored, zero failures across 310 suites in serial mode; warning-denied all-target Clippy, rustfmt, diff check, and the 300-line layout check passed.
- Lease-race regression follow-up: 3 retry unit tests, 3 coordinator workflow tests, 2 source-layout tests, and both CLI workflow tests passed alone; all-target warning-denied Clippy, rustfmt, and diff check passed. Full post-change suite is not yet verified: one initial `DirectoryNotEmpty` cleanup failure passed in isolation, and a subsequent run was stopped at 682 tests / 221 suites after stalling.
- Exact package `b298ae79774e` passed package validation and `--help`. Its native UI showed the recorded fixture coordinator, enforced empty/nonempty send-button state, and retained the candidate at human approval with 3/3 tasks and tests complete. No request was sent; the saved profile is configured for planning only. Windows UI verification remains open.
- The parallel full-suite attempt had one existing settings-dialog visibility test failure; the test passed alone and in the serial full suite. Exact packaged UI and Windows validation remain open issue #9 gates.

## Related assets

- [[native-inspector-and-judgments]]
- [[native-room-failure-feedback]]
- [[typed-coordinator-amendments]]
