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

Run `41456152913` failed after Resume with state `stopped`, no report error, and task attempt 1. Run `41465909814` passed Windows build and package smoke but failed its exact-package UI test with the same stopped state; the timeout reported `UI actions=[]`, proving no action child was spawned. Artifact `13157855546` has digest `sha256:34ad3f466abf4d50c24d692e305bcb38d78d61d53492b187912f0c6d31422799`.

The root cause was a stale revision precondition: the coordinator can finalize the stopped report after the UI snapshot was rendered, and Resume was rejected before launching its child. A new integration regression reproduces this by updating the report after snapshot creation. The fix rebases only a Resume action against a freshly loaded `stopped` report; other stale actions remain rejected, and the child validates the current stop digest. Local focused tests pass, but the updated Windows package has not yet been verified. No Windows package acceptance credit is added.

## Evidence

- [Raw lifecycle finding](../raw/2026-11-02-windows-lifecycle-resume-state.md)
- [Issue #9 implementation checklist](../../docs/07-implementation-plan.md)
- [Test harness](../../tests/windows_run_lifecycle.rs)
