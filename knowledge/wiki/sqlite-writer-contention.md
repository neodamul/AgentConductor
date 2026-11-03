# SQLite writer contention in parallel tasks

## Symptom

The exact Windows package smoke on run `39309528729` started three independent tasks against one run database. Two completed while one failed with `database is locked`, so verification and merge did not run.

## Reproduction and cause

`tests/state_contention.rs` holds an SQLite `BEGIN IMMEDIATE` lock for 2.5 seconds while another run connection writes an audit-backed document. The prior two-second `busy_timeout` fails before the lock clears. SQLite's busy handler already provides bounded waiting; transaction replay or a new writer service is unnecessary.

## Decision

Use a shared ten-second busy timeout for writable and read-only run connections. Preserve the existing transaction boundaries, commit behavior, and visible error after timeout. Keep the state implementation split into files of at most 300 lines.

## Verification

The regression failed before the timeout change and passed after it, including a readback of the committed document. CI run `39309528729` passed macOS package smoke but exposed the Windows failure; rerun exact package verification on both systems after the fix.

## Rust migration parity — 2026-10-20

The #9 branch reproduced the 2.5-second writer-lock failure, then passed the regression with the shared ten-second timeout in both connection paths. Its complete serialized suite passed 726 tests, 14 ignored, and 0 failed across 250 targets; all-target Clippy (`-D warnings`), rustfmt, diff checks, and the 300-line guard passed. The exact optimized release binary SHA-256 is `9313d3adfa62682d1b85930eaffe9956ca364dc352587c586204ae19282d8562`. The main and Rust branches retain their own original verification records.
