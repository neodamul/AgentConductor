# Task timeout retry and blocked feedback

GitHub #44 addresses a run that appeared unexplained when a long-running agent subprocess timed out. A transient wall-timeout should preserve its work and evidence so the operator can understand and recover it.

## Behavior

- A worker wall-timeout increments `TaskReport.timeouts` and retains its attempt directory, process ownership record, stdout/stderr, and partial workspace.
- The timed-out task becomes `blocked`; the report and attempt ownership retain the timeout message, stdout/stderr, and partial workspace.
- A blocked task does not relaunch automatically. The operator reviews the preserved evidence and explicitly resumes the run; that creates a new attempt while retaining prior artifacts.
- If the run ends with a blocked task, finalization copies that task's cause into `Report.error` for the same banner and report consumers.

## Verification

The `tests/workflow.rs` regression covers a worker timing out, remaining blocked with ownership evidence, then completing only after explicit resume. Keep this test when changing scheduler timeout or resume semantics.

## Architecture trace

R-153 → BG-108 → SF-140 → UC-143 → FR-138 → QAS-149 → DD-163. GitHub #44, 2026-10-19.
