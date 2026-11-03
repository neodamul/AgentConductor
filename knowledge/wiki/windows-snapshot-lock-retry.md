# Windows snapshot locking recovery

## Symptom

Exact-head Platform Validation run `41352601950` passed macOS but failed Windows
in `native_schedule_action_reaches_controller_and_persists_manual_time`. The
test scheduled a waiting task, then polled a native Room snapshot while the UI
action ran. The snapshot returned SQLite's `locking protocol` error at
`tests/native_ui_task_scheduling.rs:229`. The Windows native-package job was
skipped as a result.

## Behavior

Only `snapshot` requests are retried, because they are read-only observations
and can safely be recomputed. A snapshot retries errors containing SQLite's
`locking protocol`, `database is locked`, or `database is busy` messages, with
delays of 10, 25, 50, and 100 ms. The total is limited to five attempts. Other
request kinds are still executed once, and other errors are returned
immediately. This does not retry an action or hide a persistent lock failure.

## Regression coverage

`tests/request_diagnostics.rs` verifies recovery after two transient lock
errors, the exact backoff sequence, the five-attempt limit, and immediate
return for unrelated errors. The first run failed before the retry behavior was
implemented; the focused test and `native_ui_task_scheduling` suites then
passed 11 tests on macOS. The locked/offline all-target native suite now passes
1,281 tests with 16 ignored and 0 failures across 394 suites. Warning-denied
Clippy, formatting, source layout, and public-doc checks also pass. Exact
Platform Validation run `41356140816` now passes macOS and Windows, including
the Windows schedule regression and native UI launch path. The exact-head local
macOS package smoke passes at `e7c51826` (SHA-256
`bbd4f66b15221532525308f096fc3909a9e28823ac675287658ecee8fbfaa93f`). The
macOS package job passed. Windows run `41360959648` then showed that the chat
journeys still failed when serialized: Windows denied process breakaway inside
the GitHub host Job Object. Test serialization is retained for shared desktop
focus, while the exact package now uses an interactive scheduled-task launcher.
See [Windows packaged chat isolation](windows-packaged-chat-serialization.md)
and [the launch-context diagnosis](windows-packaged-chat-launch-context.md).

## Operational boundary

The retry is bounded below 200 ms and occurs in the native request worker. It
does not add polling to the UI thread. A repeated lock error remains visible as
a request error after the final attempt.
