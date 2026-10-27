# Reviewed Scope-Failure Recovery

An out-of-scope integration failure leaves the run stopped and preserves quarantined files. Recovery changes only one task's owned paths or test command after validating the revised plan against other tasks.

The workflow exposes a read-only failure report and a proposal preview. The preview includes previous and proposed task settings, recoverable quarantined paths, destination conflicts, and a digest of the exact proposal. Applying recovery requires that same digest, rechecks the proposal under the workflow lock, refuses path traversal and symlinked parent paths, and restores files with exclusive creation so an existing destination is never overwritten. Original quarantine evidence remains available. The run stays stopped and prior integration approval is revoked; the operator must explicitly resume it.

The focused Rust regression suite is `tests/scope_recovery.rs`. It covers scope preservation after feedback, stale proposal rejection, task overlap, safe restore, destination conflicts, symlink parents, and traversal paths. The shared workflow API is implemented in `src/workflow/scope_recovery.rs`; the native eframe review and controller integration remain tracked under GitHub issues #122 and #9 until their end-to-end gates pass.

The active Rust branch includes an eframe review window and the revision-bound local-controller/CLI route. Rendered component tests cover the entry button, editable fields, quarantine detail, approval controls, and stable page geometry. A focused pointer interaction test now drives the real Preview changes and Approve and apply buttons through hover, press, and release, then checks the exact task, paths, test argv, and reviewed digest in the emitted events. Cross-platform and exact packaged-workflow acceptance remain pending; this article does not claim those gates have passed.

## Upstream v0.1.34 integration — 2026-10-22

The Rust UI consumes the Room recovery preview and approval packets through the local controller/CLI schema. Direct Resume is refused for an out-of-scope failure and shows localized guidance to review the recovery proposal. The end-to-end Room test verifies failure → preview → explicit approval → completed job while preserving the stopped report state. It uses an RAII fixture to remove its temporary repository on both success and failure.

Verification on the integration branch: all-target native-feature tests pass (857 passed, 14 ignored, 293 suites), with focused recovery suites, source-layout, public-doc, formatting, and warning-denied Clippy checks. Packaged workflow and cross-platform acceptance remain open; this test evidence does not close those gates.
