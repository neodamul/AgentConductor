# Windows release package job timeout

## Evidence

- Platform validation run: `41438654168`, head `ecc4525f0f658be7935175e5688b6e51d74513b0`.
- The macOS and Windows `validate` jobs passed. The macOS `native-package` job also passed.
- Windows package job `125267738777` started at `2026-11-02T12:10:47Z`. Its job timeout was 25 minutes. The log records cancellation during the serialized exact-package UI test step at `2026-11-02T12:36:01Z`; this is consistent with the job timeout. The job never reached a terminal test result.
- Before cancellation, the packaged Windows lifecycle, briefing, and candidate-review tests each passed. Coordinator-chat background-worker cases were skipped because the runner denied process breakaway from its host Job Object; production safety refusal stayed enabled.

## Test-first fix

- Added `windows_package_timeout_covers_the_full_release_build_and_gui_suite` to `tests/windows_package_workflow.rs`. It failed with the old 25-minute limit.
- Changed `.github/workflows/platform-validation.yml` so `native-package` has a 45-minute timeout, matching `validate`.
- Local Green: all 6 `windows_package_workflow` tests and both `source_layout` tests pass; formatting and diff checks pass.

## Remaining

The 45-minute timeout has not yet been verified by a new exact-head package run. Do not count the cancelled run as Windows package acceptance. Raw Actions log: https://github.com/neodamul/AgentConductor/actions/runs/41438654168.
