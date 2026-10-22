# GitHub #165 coordinator feedback verification

Date: 2026-10-23

The Rust branch now has a coordinator-specific request action and draft in the native Agent Room. The path is UI → revision-bound Rust action → local `coordinator-request` subprocess → existing frozen-evidence reviewer → deterministic amendment/charter validation → audit-derived coordinator judgments. Task feedback remains unchanged. A request is bounded to 8 KiB, includes the complete bounded operator message in the audit, and never grants approval or merge authority.

TDD evidence:

- Red: focused workflow regression failed because `review_request` was not exported.
- Green: lifecycle tests verify queued/processing/completed, visible failure, retained task attempts, and refusal to merge.
- Render regression verifies the named recipient, processing status, and coordinator send control.
- Validation rejects blank, NUL-containing, and over-limit requests.
- Full local run after the lease-wait refinement: 920 passed, 14 ignored, 0 failed, 310 suites, serial execution, 369.64 seconds.
- `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `git diff --check`, and the 300-line source layout test passed.
- A settings-dialog visibility test failed once in the parallel full-suite run, then passed alone and in the final serial full suite.
- The running local v0.1.36 app was not replaced. Packaged UI and Windows end-to-end checks are still outstanding for issue #9.

## Follow-up: coordinator room-lease race

Claude's review found a race after the initial room-lease check: the room can reacquire its lease before the review worker reads the snapshot. The coordinator request now retries only errors containing `lease is held`, sleeping 100 ms between attempts and sharing the original bounded request deadline. Other review errors still fail immediately; timeout remains a recorded failed request.

- Red/green tests cover a transient lease conflict that later succeeds, immediate propagation of unrelated errors, and deadline expiry after retries.
- Focused verification: the 3 retry tests, 3 coordinator workflow tests, 2 source-layout tests, and the 2-test CLI workflow suite passed. Warning-denied all-target Clippy, rustfmt, and `git diff --check` passed.
- Full-suite verification is pending: the first post-change run had one `DirectoryNotEmpty` failure in `binary_schedules_reassigns_and_selects_models_with_native_commands`; that test and its whole suite passed when run alone. A second full run stalled after 682 tests / 221 suites and was stopped. Do not report the post-change full suite as passing.
- Exact packaged interaction and Windows UI checks remain open issue #9 gates.
