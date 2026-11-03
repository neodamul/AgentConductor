# Task timeout retry and Stop-aware backoff

GitHub #44 addresses a run that appeared unexplained when a long-running agent subprocess timed out. The coordinator must report the timeout, preserve its work and evidence, and retry automatically without leaving the task blocked.

## Behavior

- A worker wall-timeout increments `TaskReport.timeouts` and retains its attempt directory, process ownership record, stdout/stderr, and partial workspace.
- The scheduler marks the task pending and sets a retry deadline using a bounded delay (`min(task timeout, 60 seconds)`). The Unix-second deadline is rounded up so the retry cannot start before the requested delay.
- The scheduler waits for that deadline while checking the human Stop request. Stop cancels the pending retry and preserves the completed attempt record; otherwise the worker starts a new attempt and keeps earlier artifacts.
- Manual Resume remains available for a stopped or otherwise blocked run. A successful retry proceeds to integration verification and human approval.
- The automatic timeout retry currently has no attempt ceiling; a repeatedly timing-out agent may continue consuming resources until a person stops the run.

## Verification

The `tests/workflow.rs` regressions cover a worker timing out and succeeding on an automatic retry, plus a Stop request that interrupts timeout backoff before relaunch. They assert the timeout count, prior ownership evidence, final task state, and attempt count. Keep these tests when changing scheduler timeout or resume semantics.

## Architecture trace

R-153 → BG-108 → SF-140 → UC-143 → FR-138 → QAS-149 → DD-163. GitHub #44, 2026-10-19; native Rust parity ported on issue #9, 2026-10-20.
