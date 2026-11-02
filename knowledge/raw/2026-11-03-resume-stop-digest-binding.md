# UI Resume must bind to the reviewed stop marker

## Decision

Retain the stale-revision exception for Resume only when the stopped-run UI request carries the exact stop-marker digest displayed by its snapshot. The controller validates that digest against the current marker even when the report revision still matches, then forwards the same digest to the child process.

## Reason

The previous controller rebased any stale Resume request while the latest report remained `stopped`. A stale UI action could therefore clear a newer stop request. This violated DD-22, which requires explicit Resume to bind the previously reviewed stop marker and forbids clearing a newer request.

## Evidence

The TDD regression `stale_resume_cannot_clear_a_newer_stop_request` failed before the fix because the stale action was accepted. After the fix, local verification passed: 1,296 tests passed, 16 ignored across 403 suites; 16 focused UI-resume/native-launch/layout tests passed; warning-denied all-target Clippy passed. Exact-head macOS/Windows and packaged Windows UI verification remain pending.
