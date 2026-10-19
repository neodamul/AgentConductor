# Task timeout retry and blocked feedback

GitHub #44 addresses a run that appeared unexplained when a long-running agent subprocess timed out. A transient wall-timeout should preserve its work and evidence so the operator can understand and recover it.

## Behavior

- A worker wall-timeout increments `TaskReport.timeouts` and retains its attempt directory, process ownership record, stdout/stderr, and partial workspace.
- The scheduler records `task_timeout_resume`, keeps the task pending, and schedules a retry after `min(task timeout, 60 seconds)`. Each attempt receives a fresh artifact directory while continuing from retained files/session context.
- The report exposes the timeout message while the retry is pending; the existing run banner reads the report-level error.
- The scheduler checks `stop.request` before launching pending tasks. A Stop during backoff therefore ends the run without starting another attempt.
- If the run ends with a blocked task, finalization copies that task's cause into `Report.error` for the same banner and report consumers.

## Verification

The `tests/workflow.rs` regressions cover a worker that times out once then completes, including retained first-attempt ownership evidence, and a Stop request during the backoff that prevents a second attempt. Keep both tests when changing scheduler retry or stop semantics.

## Architecture trace

R-153 → BG-108 → SF-140 → UC-143 → FR-138 → QAS-149 → DD-163. GitHub #44, 2026-10-19.
