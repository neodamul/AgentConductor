# Failed integration output streams

## Contract

A failed integration report is also the source for the operator's retry feedback. Selecting the first nonempty process stream is unsafe: a tooling warning in stderr can hide the test error in stdout.

## Rust implementation

`src/ui.rs::public_report` reads the existing bounded tail (2 KiB) from both streams. If both contain text, it returns labeled `stderr` and `stdout` sections, with stdout last. If only one stream is readable and nonempty, that text is returned unchanged. No severity heuristics, extra process, polling, or full-log read is added.

The native Room snapshot carries this combined `output` into the failed-task details and the generated retry instruction.

## Evidence

GitHub #323. Test-first integration coverage in `tests/ui_failure_report.rs` proves both messages and stream order through a real `Room::snapshot`; `tests/native_run_status_banner.rs` proves the combined details reach the retry-feedback UI. Single-stream compatibility is covered as well. Focused verification: 20 passed across six suites, warning-denied Clippy, rustfmt, diff checks, and the 300-line source/test guard. The code is local on the root #9 worktree until its current platform validation finishes; exact packaged Windows verification remains pending.
