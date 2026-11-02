# Exact Windows lifecycle failure and diagnostic follow-up

## Source

- Run: https://github.com/neodamul/AgentConductor/actions/runs/41446760915
- PR head: `05a2fd748455d5eaa877c4694fefef9c130e6bc0`
- Job: Windows `native-package`, job `125295536348`
- Artifact: `13145767282`, digest `sha256:c123fdc30dc7016b4f2eb04b667e30107a8d371f45e763e7451e362a76bb29ba`
- AgentMemory: `mem_mvzb5uaw_b64297ed9957`

## Observed outcome

Both platform-validation jobs and the macOS package job passed. The Windows package job reached its exact-binary UI suite and completed under the configured 45-minute limit. Windows package build, manual rendering, package execution, default desktop save, briefing, candidate review, and run recovery passed. The run-lifecycle test failed because its persisted report did not reach `waiting_approval` within 30 seconds after the UI Resume action. Three coordinator-chat lifetime cases were explicitly skipped because the hosted runner denied Job Object breakaway; the production refusal remained enabled.

This disproves the theory that the earlier cancelled run was only a Windows job-timeout problem. The current run exposes a distinct stop/resume acceptance failure. Packaged workflow acceptance remains 22/25 and shipped/default Rust UI acceptance remains 0/25.

## TDD diagnostic follow-up

The old failure message gave only the expected state and discarded the observed report before temporary cleanup. `tests/windows_run_lifecycle.rs` now prints the saved run state, error, and task status/attempt counts when the expected state times out. Local `windows_run_lifecycle` and `source_layout` suites pass 3 tests, with the exact-package test ignored on macOS. This change does not alter production code or establish Windows acceptance. The next exact Windows package run must reveal the persisted failure reason before a product fix is selected.

## UI-action process diagnostic follow-up

- Run `41456152913` showed `stopped`, no report error, and task attempt 1 after Resume. The test still could not tell whether the UI rejected the request before spawning a child or the child failed.
- Commit `45f65ed71516c1d932f7d457122cc1f37eaa4316` makes a timeout print persisted `coordinator-ui-*/process/owner.json` and stdout/stderr tails before test cleanup.
- Local focused test contract, warning-denied test Clippy, rustfmt, diff check, and the 298-line limit pass. No product acceptance credit is added.
- A new local `ui_resume` integration test drives `Room::dispatch` through the broker child and passes on macOS: the stopped task resumes to `waiting_approval` at attempt 2. This rules out the general controller/child path on macOS, but does not test Windows UI Automation or the Windows release package.
- The test waits for its matching UI child job to reach `completed`/exit code 0 before deleting the fixture. The run report can reach `waiting_approval` just before the child writes its audit result; removing the fixture earlier caused a `DirectoryNotEmpty` race. The strengthened test passed three consecutive focused runs.
- Exact-head run `41465909814` passed macOS and Windows platform validation, Windows Release build/package smoke, and the macOS arm64 package (archive SHA-256 `5a1815c2ac7b044db467414459c38b0f0b4f6df459f7d773e7af9bbe4cd424ca`, artifact `13157010142`). The Windows package UI suite failed: after clicking Resume, the report remained `stopped` at attempt 1 and the test found no spawned UI-action directories (`UI actions=[]`). The exact Windows artifact is `13157855546`, digest `sha256:34ad3f466abf4d50c24d692e305bcb38d78d61d53492b187912f0c6d31422799`.
- Root cause reproduced locally: if the stopped report is finalized after the UI rendered its snapshot, Resume is rejected on the stale report revision before a UI child is launched. The controller now permits only Resume on a freshly loaded `stopped` report to use its current revision; other stale actions remain rejected. The resumed CLI still validates the current stop digest. Full local `cargo test --offline --all-targets` passes (1,295 passed, 16 ignored across 403 suites in 290.93s); focused UI-resume/layout/knowledge passes (7); all-target warning-denied Clippy, formatting, and diff checks pass. Exact packaged Windows verification remains required.
