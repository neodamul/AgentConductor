# SQLite writer contention in parallel tasks

## Symptom

The exact Windows package smoke on run `39309528729` started three independent tasks against one run database. Two completed while one failed with `database is locked`, so verification and merge did not run.

## Reproduction and cause

`tests/state_contention.rs` holds an SQLite `BEGIN IMMEDIATE` lock for 2.5 seconds while another run connection writes an audit-backed document. The prior two-second `busy_timeout` fails before the lock clears. SQLite's busy handler already provides bounded waiting; transaction replay or a new writer service is unnecessary.

## Decision

Use a shared ten-second busy timeout for writable and read-only run connections. Preserve the existing transaction boundaries, commit behavior, and visible error after timeout. Keep the state implementation split into files of at most 300 lines.

## Verification

The regression failed before the timeout change and passed after it, including a readback of the committed document. CI run `39316534644` passed macOS and Windows validation and exact package smoke, including the parallel workflow. Local macOS package evidence is stored at `docs/evidence/issue56-macos-package.json` on main.
