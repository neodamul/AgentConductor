# SQLite writer contention in parallel tasks

## Symptom

The exact Windows package smoke on run `39309528729` started three independent tasks against one run database. Two completed while one failed with `database is locked`, so verification and merge did not run.

## Reproduction and cause

`tests/state_contention.rs` holds an SQLite `BEGIN IMMEDIATE` lock for 2.5 seconds while another run connection writes an audit-backed document. The prior two-second `busy_timeout` fails before the lock clears. SQLite's busy handler already provides bounded waiting; transaction replay or a new writer service is unnecessary.

## Decision

Use a shared ten-second busy timeout for writable and read-only run connections. Preserve the existing transaction boundaries, commit behavior, and visible error after timeout. Keep the state implementation split into files of at most 300 lines.

## Verification

The regression failed before the timeout change and passed after it, including a readback of the committed document. All 182 default-feature Rust test suites pass (548 passed, 0 failed, 14 ignored), with Clippy warnings denied. The exact macOS package from source commit `39071889592e23281110e0157a3e95893deff3c2` passed all four shared/worktree × Git/folder smoke cases; its archive SHA-256 is `7c5930b7fec85bba72e45ee3574beb9f077bfb5e9c091d4e713a133fe0945fa4`. CI run `39309528729` exposed the Windows failure; rerun exact package verification on the issue-linked PR.
