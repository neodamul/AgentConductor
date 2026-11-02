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
