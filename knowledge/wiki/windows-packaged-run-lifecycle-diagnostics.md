# Windows packaged run lifecycle diagnostics

## Preserve evidence before fixture cleanup

The exact Windows package lifecycle test owns a temporary run directory. When `wait_state` times out, the assertion must report both persisted workflow state and every `coordinator-ui-*/process` record before the test fixture removes the directory.

Each UI action process directory may contain:

- `owner.json`: launch phase, child PID, and terminal exit code.
- `stdout.log` and `stderr.log`: the broker action's CLI result or failure.

Use these together with the workflow report's state, error, task status, and attempt count. An absent action directory means the controller did not persist a child launch; a present terminal record and its logs show whether a launched action completed or failed. Do not infer the product root cause from the button's UI Automation `Invoke` returning successfully; that only confirms the automation request was submitted.

The test keeps the evidence bounded by including only the last 2 KiB of each output stream. Its temporary fixture still cleans up after the failure has been printed.

When testing the broker child, wait for the matching snapshot job to reach `completed` with exit code 0 before removing the run fixture. The persisted workflow can reach `waiting_approval` just before the UI child writes its audit-result record; deleting the directory at that point can race with the child and fail with `DirectoryNotEmpty`.

## Current trace

Run `41456152913` failed after Resume with state `stopped`, no report error, and task attempt 1. A local integration test now drives `Room::dispatch` through the broker child and reaches `waiting_approval` on attempt 2 on macOS. This rules out the general controller/child resume path on macOS; Windows UI Automation and the packaged Windows executable remain unverified.

Commit `45f65ed7` adds action-process evidence to the Windows timeout assertion. Run `41465909814` passed macOS and Windows platform validation, including Windows all-target native tests, opt-in Rust UI launch, formatting, Clippy, and unsafe-block checks. Both package jobs are running. This is not a product fix and does not add package acceptance credit.

## Evidence

- [Raw lifecycle finding](../raw/2026-11-02-windows-lifecycle-resume-state.md)
- [Issue #9 implementation checklist](../../docs/07-implementation-plan.md)
- [Test harness](../../tests/windows_run_lifecycle.rs)
